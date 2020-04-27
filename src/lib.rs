use num;
use std::fmt;

use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;

// pub trait MxElement: num::Unsigned + num::PrimInt + std::fmt::Debug {
pub trait MxElement:
    std::ops::AddAssign
    + num::One
    + num::Unsigned
    + num::PrimInt
    + std::cmp::PartialOrd
    + std::fmt::Debug
    + std::fmt::Display
{
    // type Range: Iterator<Item = Self>;
}

impl MxElement for u8 {
    //     type Range = std::ops::Range<u8>;
}

impl MxElement for u16 {
    // type Range = std::ops::Range<u16>;
}
impl MxElement for u32 {
    // type Range = std::ops::Range<u32>;
}

impl MxElement for usize {
    // type Range = std::ops::Range<usize>;
}

#[cfg(target_pointer_width = "64")]
impl MxElement for u64 {
    // type Range = std::ops::Range<u64>;
}

fn compress<T>(row: Vec<T>, col: Vec<T>, n: usize) -> (Vec<usize>, Vec<T>)
where
    T: MxElement,
{
    // println!("n = {}", n);
    let mut w: Vec<usize> = vec![0; n];
    let mut ja: Vec<T> = vec![T::zero(); col.len()];

    // println!("length of w = {}", w.len());
    for v in &row {
        // println!("setting w[{:?}]", v);
        w[v.to_usize().expect("Invalid vertex entry")] += 1;
    }
    let ia = w.iter().fold(vec![0], |mut acc, val| {
        acc.push(val + acc.last().unwrap());
        acc
    });
    let mut w = ia.clone();
    if let Some(last) = w.last_mut() {
        *last = 0;
    }
    // println!("w = {:?}", w);
    // println!("ia = {:?}", ia);
    for (j, v) in col.into_iter().enumerate() {
        let rj = row[j].to_usize().expect("Invalid vertex entry");
        let p = w[rj];
        // println!("rj = {}, p = {}, w = {:?}", rj, p, w);
        ja[p] = v;
        w[rj] += 1;
    }
    // println!("w = {:?}", w);

    (ia, ja)
}

#[derive(Debug, Clone)]
pub struct GraphMatrix<T>
where
    T: MxElement,
{
    indptr: Vec<usize>,
    indices: Vec<T>,
}

impl<T> GraphMatrix<T>
where
    T: MxElement,
{
    pub fn from_edges(edgelist: Vec<(T, T)>) -> Self {
        // -> GraphMatrix<T, U>  {
        // assume edgelist is not sorted
        let mut sorted_edge_list = edgelist;
        sorted_edge_list.sort_unstable();
        let (ss, ds): (Vec<_>, Vec<_>) = sorted_edge_list.into_iter().unzip();

        // println!("ss = {:?}, ds = {:?}", ss, ds);
        let m1 = ss.last().expect("Edgelist must not be empty");
        let m2 = ds.iter().max().expect("Edgelist must not be empty");
        let m = m1
            .max(m2)
            .to_usize()
            .expect("Number of vertices must be less than 2^64-2")
            + 1;
        // println!("m1 = {:?}, m2 = {:?}, m = {}", m1, m2, m);
        let (indptr, indices) = compress(ss, ds, m);
        GraphMatrix { indptr, indices }
    }
    // we don't want to consume self, and we need to ensure that the iterator lasts as long as the
    // struct. Shorthand for this is pub fn row(&self, r: usize) -> impl Iterator<Item = T> + '_.
    // pub fn row<'a>(&'a self, r: usize) -> impl Iterator<Item = T> + 'a
    pub fn row(&self, r: T) -> &[T]
    where
        T: MxElement,
    {
        let ru = r.to_usize().unwrap();
        if ru > self.indptr.len() - 1 {
            panic!("Row {} is out of bounds (max {})", r, self.indptr.len() - 1)
        }

        let row_start = unsafe { self.indptr.get_unchecked(ru) };
        let row_end = unsafe { self.indptr.get_unchecked(ru + 1) };

        &self.indices[*row_start..*row_end]
    }

    pub fn row_len(&self, r: usize) -> T
    where
        T: MxElement,
    {
        if r > self.indptr.len() - 1 {
            panic!("Row {} is out of bounds (max {})", r, self.indptr.len() - 1)
        }
        let row_start = self.indptr[r];
        let row_end = self.indptr[r + 1];
        T::from(row_end - row_start).expect("Something went wrong.")
    }

    pub fn has_index(&self, r: T, c: T) -> bool
    where
        T: MxElement,
    {
        let row = self.row(r);
        let tc = T::from(c).expect("invalid vertex");
        row.binary_search(&tc).is_ok()
    }

    pub fn from_edge_file(fname: &Path) -> Self {
        let f = File::open(fname).expect("Cannot open file");
        let file = BufReader::new(&f);
        let mut edgelist: Vec<(T, T)> = vec![];
        for line in file.lines() {
            let l = line.expect("error reading file"); // produces a std::string::String
            let l = l.trim(); // changes to &str
            if l.starts_with("#") {
                continue;
            }
            let mut eit = l.split_whitespace();
            let s1 = eit.next().expect("Invalid line (first field)");
            let s2 = eit.next().expect("Invalid line (second field)");
            if eit.next().is_some() {
                panic!("Invalid line (extra fields)");
            }

            let src128: u128 = s1.parse().unwrap();
            let dst128: u128 = s2.parse().unwrap();

            let src = T::from(src128).expect("vertex out of range");
            let dst = T::from(dst128).expect("vertex out of range");
            edgelist.push((src, dst));
        }
        GraphMatrix::from_edges(edgelist)
    }

    // return the number of defined values
    pub fn n(&self) -> usize {
        self.indices.len()
    }

    pub fn dim(&self) -> usize {
        self.indptr.len() - 1
    }
}

impl<T> fmt::Display for GraphMatrix<T>
where
    T: MxElement,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "GraphMatrix {} x {} with {} entries",
            self.dim(),
            self.dim(),
            self.n()
        )
    }
}
