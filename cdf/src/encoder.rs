use core::borrow::Borrow;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, Write};
use std::path::Path;

use crate::{
    AtomicConfig, Config, Constraint, Context, ContextUnit, Element, FixedText, Preamble, Source,
    Witness,
};

/// An encoder for CDF format
#[derive(Debug)]
pub struct Encoder<WI, CI, T> {
    preamble: Preamble,
    witnesses: WI,
    constraints: CI,
    target: T,
}

impl<WI, CI, T> Encoder<WI, CI, T> {
    /// Create a new encoder
    pub(crate) const fn new(preamble: Preamble, witnesses: WI, constraints: CI, target: T) -> Self {
        Self {
            preamble,
            witnesses,
            constraints,
            target,
        }
    }

    /// Return the inner target
    pub fn into_inner(self) -> T {
        self.target
    }
}

impl<W, WI, C, CI, T> Encoder<WI, CI, T>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
{
    /// Intialize a preamble from a witness & constraints iterator
    pub fn init_preamble(config: Config, witnesses: &WI, constraints: &CI) -> Preamble {
        *Preamble::new()
            .with_config(config)
            .with_witnesses(witnesses.len())
            .with_constraints(constraints.len())
    }
}

impl<W, WI, C, CI> Encoder<WI, CI, File>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
{
    /// Initialize the encoder, filling a file with required bytes.
    ///
    /// Check [`File::set_len`]
    pub fn init_file<P>(config: Config, witnesses: WI, constraints: CI, path: P) -> io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let preamble = Self::init_preamble(config, &witnesses, &constraints);
        let len = preamble.source_cache_offset(0);

        let file = OpenOptions::new().write(true).create(true).open(path)?;

        file.set_len(len as u64)?;

        Ok(Self::new(preamble, witnesses, constraints, file))
    }
}

impl<W, WI, C, CI, B> Encoder<WI, CI, io::BufWriter<B>>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
    B: io::Write + io::Seek,
{
    /// Initialize the encoder, filling the buffer with required bytes.
    pub fn init_buffer(
        config: Config,
        witnesses: WI,
        constraints: CI,
        buffer: B,
    ) -> io::Result<Self> {
        let preamble = Self::init_preamble(config, &witnesses, &constraints);
        let len = preamble.source_cache_offset(0);

        let mut buffer = io::BufWriter::new(buffer);

        let n = buffer
            .rewind()
            .and_then(|_| buffer.write(&vec![0u8; len]))?;

        if n != len {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "the target wrote {} bytes instead of the expected {}!",
                    n, len
                ),
            ));
        }

        Ok(Self::new(preamble, witnesses, constraints, buffer))
    }
}

impl<W, WI, C, CI> Encoder<WI, CI, io::Cursor<Vec<u8>>>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
{
    /// Initialize the encoder, filling the cursor with required bytes.
    pub fn init_cursor(config: Config, witnesses: WI, constraints: CI) -> Self {
        let preamble = Self::init_preamble(config, &witnesses, &constraints);
        let len = preamble.source_cache_offset(0);

        let bytes = vec![0u8; len];
        let cursor = io::Cursor::new(bytes);

        Self::new(preamble, witnesses, constraints, cursor)
    }
}

impl<W, WI, C, CI, T> Encoder<WI, CI, T>
where
    W: Borrow<Witness>,
    WI: Iterator<Item = W> + ExactSizeIterator,
    C: Borrow<Constraint>,
    CI: Iterator<Item = C> + ExactSizeIterator,
    T: io::Write + io::Seek,
{
    // this function is internal so this excessive args count won't leak to the API
    #[allow(clippy::too_many_arguments)]
    fn write_iter<B, E, I, FID, FP, FO>(
        config: &<E as Element>::Config,
        preamble: &Preamble,
        source_cache: &mut HashMap<FixedText<{ Source::PATH_LEN }>, usize>,
        context: &mut ContextUnit,
        n: usize,
        mut iter: I,
        fid: FID,
        fp: FP,
        fo: FO,
        target: &mut T,
    ) -> io::Result<usize>
    where
        B: Borrow<E>,
        E: Element,
        I: Iterator<Item = B>,
        FID: Fn(&E) -> usize,
        FP: Fn(&E) -> FixedText<{ Source::PATH_LEN }>,
        FO: Fn(&Preamble, usize) -> Option<usize>,
    {
        iter.try_fold(n, |n, l| {
            let l = l.borrow();
            let id = fid(l);

            let path = fp(l);
            let source_id = source_cache.len();
            let source_id = *source_cache.entry(path).or_insert(source_id);

            let ctx = context.with_source_cache_id(source_id);

            fo(preamble, id)
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to calculate offset"))
                .map(|ofs| io::SeekFrom::Start(ofs as u64))
                .and_then(|ofs| target.seek(ofs))
                .and_then(|_| l.try_to_writer(target.by_ref(), config, ctx))
                .map(|x| n + x)
        })
    }

    /// Write all witnesses and constraints into the target
    pub fn write_all(&mut self) -> io::Result<usize>
    where
        W: Borrow<Witness>,
        WI: Iterator<Item = W>,
        C: Borrow<Constraint>,
        CI: Iterator<Item = C>,
    {
        let preamble = &self.preamble;
        let config = &self.preamble.config;
        let witnesses = &mut self.witnesses;
        let constraints = &mut self.constraints;
        let target = &mut self.target;

        let context = &mut Context::unit();
        let mut source_cache: HashMap<FixedText<{ Source::PATH_LEN }>, usize> = HashMap::new();

        let n = preamble.try_to_writer(target.by_ref(), &AtomicConfig, context)?;

        let n = Self::write_iter(
            config,
            preamble,
            &mut source_cache,
            context,
            n,
            witnesses,
            |w| w.id(),
            |w| w.source().path().clone(),
            Preamble::witness_offset,
            target,
        )?;

        let n = Self::write_iter(
            config,
            preamble,
            &mut source_cache,
            context,
            n,
            constraints,
            |c| c.id(),
            |c| c.source().path().clone(),
            Preamble::constraint_offset,
            target,
        )?;

        let mut source_cache: Vec<(FixedText<{ Source::PATH_LEN }>, usize)> =
            source_cache.into_iter().collect();

        source_cache.as_mut_slice().sort_by_key(|x| x.1);

        // Move the cursor to the logical end of the file
        let ofs = preamble.source_cache_offset(0);
        let ofs = io::SeekFrom::Start(ofs as u64);

        target.seek(ofs)?;

        let n = source_cache.into_iter().try_fold(n, |n, (path, _idx)| {
            path.try_to_writer(target.by_ref(), config, context)
                .map(|x| x + n)
                .and_then(|n| target.seek(io::SeekFrom::Start(n as u64)).map(|_| n))
        })?;

        Ok(n)
    }
}
