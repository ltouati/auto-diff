use std::fmt;

/// Naive tensor implementation, single thread
pub struct GenTensor<T> {
    d: Vec<T>,
    dim: Vec<usize>,
}
impl<T> GenTensor<T> where T: num_traits::Float {

    /// Creation Ops
    
    pub fn new() -> GenTensor<T> {
        GenTensor { d: Vec::<T>::new(), dim: Vec::new() }
    }

    /// Create a tensor with given Vec.
    pub fn new_raw(data: &[T], shape: &[usize]) -> GenTensor<T> {
        let new_data = data.to_vec();
        let new_dim = shape.to_vec();
        GenTensor {
            d: new_data,
            dim: new_dim,
        }
    }

    /// Convert 1 dim index to multi-dim index.
    pub fn index2dimpos(&self, index: usize) -> Vec::<usize> {
        if index >= self.d.len() {
            panic!("index out of range, {:?}, {:?}", index, self.d.len());
        }
        let mut ret = Vec::new();
        let mut reminder = index;
        for i in &self.stride() {
            //println!("{}", reminder);
            ret.push(reminder/i);
            reminder %= i;
        }
        ret
    }

    /// Convert multi-dim index to 1 dim index.
    pub fn dimpos2index(&self, dimpos: &[usize]) -> usize {
        if dimpos.len() != self.dim.len() {
            panic!("get expects the same dim self.dim: {:?}, o: {:?}", self.dim, dimpos);
        }
        for (i, j) in self.dim.iter().zip(dimpos.iter()) {
            if j >= i {
                panic!("get expects the dim within range self.dim: {:?}, o: {:?}", self.dim, dimpos);
            }
        }
        let mut ret = 0;
        for (st, i) in self.stride().iter().zip(dimpos.iter()) {
            //println!("{}", reminder);
            ret += st*i;
        }
        ret
    }

    // 
    // as_tensor
    // as_strided
    // from_ndarray
    // zeros
    pub fn zeros(size: &[usize]) -> GenTensor<T> {
        let cap = size.iter().product();
        GenTensor {
            d: vec![T::zero(); cap],
            dim: size.to_vec(),
        }
    }
    // zeros_like
    pub fn zeros_like(&self) -> GenTensor<T> {
        let new_data = vec![T::zero(); self.d.len()];
        let new_dim = self.dim.to_vec();
        GenTensor {
            d: new_data,
            dim: new_dim,
        }
    }

    // ones
    pub fn ones(size: &[usize]) -> GenTensor<T> {
        let cap = size.iter().product();
        GenTensor {
            d: vec![T::one(); cap],
            dim: size.to_vec(),
        }
    }
    // ones_like
    pub fn ones_like(&self) -> GenTensor<T> {
        let new_data = vec![T::one(); self.d.len()];
        let new_dim = self.dim.to_vec();
        GenTensor {
            d: new_data,
            dim: new_dim,
        }
    }
    // arange
    pub fn arange(end: usize) -> GenTensor<T> {
        let mut ret = GenTensor::<T>::empty(&vec![end]);
        for i in 0..end {
            ret.d[i] = T::from(i).expect("");
        }
        ret
    }
    // range
    // linspace
    // logspace
    // eye
    pub fn empty(shape: &[usize]) -> GenTensor<T> {
        let mut elem = 1;
        for i in shape {
            elem *= i;
        }
        
        let mut new_data = Vec::with_capacity(elem);
        unsafe{ new_data.set_len(elem); }
        let new_dim = shape.to_vec();
        GenTensor {
            d: new_data,
            dim: new_dim,
        }
    }
    // empty_like
    // empty_stided
    // full
    // full_like
    // quantize_per_tensor
    // quantize_per_channel
    // 

    /// Create a tensor filled with the same value d
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::fill(1., &vec![3,5,2]);
    /// ```
    pub fn fill(d: T, shape: &[usize]) -> GenTensor<T> {
        let mut dsize = 1;
        for i in shape {
            dsize *= *i;
        }
        GenTensor {
            d: vec![d; dsize],
            dim: shape.to_vec(),
        }
    }
    /// assign a row.
    pub fn from_record(&mut self, row: usize, record: &[f32]) -> Result<(), ()> {
        for (i, index) in record.iter().zip(0..self.dim[self.dim.len()-1]) {
            self.d[row*self.dim[self.dim.len()-1] + index] = T::from(*i).expect("");
        }
        Ok(())
    }

    /// Right dimension changes fastest.
    /// Right dimension has the stride 1.
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::new_raw(&vec![0.; 3*5*2], &vec![3,5,2]);
    /// assert_eq!(m1.stride(), vec![10,2,1]);
    /// ```
    pub fn stride(&self) -> Vec<usize> {
        let mut ret = vec![0; self.dim.len()];
        let dsize = ret.len();
        for i in 0..dsize {
            if i == 0 {
                ret[dsize-1] = 1;
            } else {
                ret[dsize-i-1] = ret[dsize-i]*self.dim[dsize-i];
            }
        }
        ret
    }
    
    /// Return value at the index of the tensor.
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::new_raw(&vec![1.,2.,3.,4.,5.,6.], &vec![2,3]);
    /// assert_eq!(m1.get(&vec![1,1]), 5.);
    /// ```
    pub fn get(&self, o: &[usize]) -> T {
        if o.len() != self.dim.len() {
            panic!("get expects the same dim self.dim: {:?}, o: {:?}", self.dim, o);
        }
        for (i, j) in self.dim.iter().zip(o.iter()) {
            if j >= i {
                panic!("get expects the dim within range self.dim: {:?}, o: {:?}", self.dim, o);
            }
        }
        let stride = self.stride();
        let dsize = o.len();
        let mut index = 0;
        //println!("{:?}", stride);
        for i in 0..dsize {
            index += stride[i]*o[i];
        }
        //println!("index: {:?}", index);
        self.d[index]
    }
    pub fn set(&mut self, o: &[usize], v: T) {
        if o.len() != self.dim.len() {
            panic!("get expects the same dim self.dim: {:?}, o: {:?}", self.dim, o);
        }
        for (i, j) in self.dim.iter().zip(o.iter()) {
            if j >= i {
                panic!("get expects the dim within range self.dim: {:?}, o: {:?}", self.dim, o);
            }
        }
        let stride = self.stride();
        let dsize = o.len();
        let mut index = 0;
        for i in 0..dsize {
            index += stride[i]*o[i];
        }
        self.d[index] = v;
    }
    pub fn set_1d(&mut self, o: usize, v: T) {
        if o < self.d.len() {
            self.d[o] = v;
        } else {
            panic!("o {} is beyond limit {}", o, self.d.len());
        }
    }
    pub fn get_mut(&mut self, o: &[usize]) -> &mut T {
        if o.len() != self.dim.len() {
            panic!("get expects the same dim self.dim: {:?}, o: {:?}", self.dim, o);
        }
        for (i, j) in self.dim.iter().zip(o.iter()) {
            if j >= i {
                panic!("get expects the dim within range self.dim: {:?}, o: {:?}", self.dim, o);
            }
        }
        let stride = self.stride();
        let dsize = o.len();
        let mut index = 0;
        for i in 0..dsize {
            index += stride[i]*o[i];
        }
        &mut self.d[index]
    }

    /// dump the underlying vec
    pub fn get_raw(&self) -> Vec<T> {
        self.d.to_vec()
    }
    pub fn get_u8(&self) -> Option<Vec<u8>> {
        let mut ret = Vec::<u8>::with_capacity(self.d.len());
        for i in &self.d {
            let val = i.to_u8()?;
            ret.push(val);
        }
        Some(ret)
    }
    
    /// dump the single value in the tensor
    /// if it is the single value in the tensor.
    pub fn get_scale(&self) -> T {
        if self.dim.len() <= 1 && self.d.len() == 1 {
            self.d[0]
        } else {
            panic!("Only one element tensor can get_scale()");
        }
    }
    
    // get NCHW elements
    /// get NCHW elements, always return the size of left most dimension.
    pub fn get_n(&self) -> GenTensor<T> {
        GenTensor {
            d: vec![T::from(self.dim[0]).expect("N")],
            dim: vec![1],
        }
    }
    /// get NCHW elements, always return the size of second left most dimension.
    pub fn get_c(&self) -> GenTensor<T> {
        GenTensor {
            d: vec![T::from(self.dim[1]).expect("N")],
            dim: vec![1],
        }
    }
    /// get NCDHW elements, will require the self.dim has 5 dimensions.
    pub fn get_d(&self) -> GenTensor<T> {
        if self.dim.len() == 5 {
            GenTensor {
                d: vec![T::from(self.dim[2]).expect("N")],
                dim: vec![1],
            }            
        } else {
            panic!("Bad shape for get_D");
        }

    }
    /// get NCDHW elements, will require the self.dim has 5 dimensions or 4 dimensions.
    pub fn get_h(&self) -> GenTensor<T> {
        if self.dim.len() == 5 {
            GenTensor {
                d: vec![T::from(self.dim[3]).expect("N")],
                dim: vec![1],
            }
        } else if self.dim.len() == 4 {
            GenTensor {
                d: vec![T::from(self.dim[2]).expect("N")],
                dim: vec![1],
            }
        } else {
            panic!("Bad shape for get_D");
        }
    }
    /// get NCDHW elements, will require the self.dim has 5 dimensions or 4 dimensions.
    pub fn get_w(&self) -> GenTensor<T> {
        if self.dim.len() == 5 {
            GenTensor {
                d: vec![T::from(self.dim[4]).expect("N")],
                dim: vec![1],
            }
        } else if self.dim.len() == 4 {
            GenTensor {
                d: vec![T::from(self.dim[3]).expect("N")],
                dim: vec![1],
            }
        } else {
            panic!("Bad shape for get_D");
        }
    }

    /// Returns the size of the self tensor.
    pub fn size(&self) -> &Vec<usize> {
        &self.dim
    }
    pub fn get_data(&self) -> &Vec<T> {
        &self.d
    }
    pub fn get_data_mut(&mut self) -> &mut Vec<T> {
        &mut self.d
    }

    /// Returns the total number of elements in the input tensor
    pub fn numel(&self) -> usize {
        self.d.len()
    }

    /// Returns the total number of elements in the input tensor
    pub fn numel_tensor(&self) -> GenTensor<T> {
        GenTensor {
            d: vec![T::from(self.d.len()).expect(""),],
            dim: vec![1],
        }
    }

    /// Return portion of the image.
    /// Every range of each dim with inclusive start and exclusive end.
    pub fn get_patch(&self, range: &[(usize, usize)], step: Option<&[usize]>) -> GenTensor<T> {
        if range.len() != self.dim.len() {
            panic!("Expect range covers all dimension range: {:?}, dim: {:?}", range, self.dim);
        }
        let mut step_dim = vec![1; self.dim.len()];
        if step.is_some() {
            step_dim = step.expect("").to_vec();
        }

        // index store the the index needs visit at each dim.
        let mut index = Vec::<Vec::<usize>>::new();
        let mut total_elem = 1;
        let mut ret_dim = Vec::new();
        for (i, dim_index) in range.iter().zip(0..self.dim.len()) {
            let mut pos = i.0;
            let mut all_index = Vec::new();
            while pos < i.1 {
                all_index.push(pos);
                pos += step_dim[dim_index];
            }
            //println!("{:?}", &all_index);
            ret_dim.push(all_index.len());
            total_elem *= all_index.len();
            index.push(all_index);
        }
        let mut ret_data = Vec::<T>::with_capacity(total_elem);
        unsafe{ ret_data.set_len(total_elem); }
        let mut ret = GenTensor {
            d: ret_data,
            dim: ret_dim,
        };

        let d = self.dim.len();
        let mut pos_index = vec![0; d];
        let mut self_index = vec![0; d];
        loop {
            //println!("pos_index: {:?}", pos_index);
            for i in 0..d {
                self_index[i] = index[i][pos_index[i]];
            }
            let value = self.get(&self_index);
            ret.set(&pos_index, value);

            for dim_index in 0..d {
                pos_index[d-1-dim_index] += 1;
                if pos_index[d-1-dim_index] >= ret.dim[d-1-dim_index] {
                    pos_index[d-1-dim_index] = 0;
                } else {
                    break;
                }
            }

            if pos_index == vec![0; d] {
                break;
            }
        }
        
        ret
    }

    pub fn set_patch(&mut self, val: &GenTensor<T>, range: &[(usize, usize)], step: Option<&[usize]>) {
        if range.len() != self.dim.len() {
            panic!("Expect range covers all dimension range: {:?}, dim: {:?}", range, self.dim);
        }

        let mut step_dim = vec![1; self.dim.len()];
        if step.is_some() {
            step_dim = step.expect("").to_vec();
        }

        // index store the the index needs visit at each dim.
        let mut index = Vec::<Vec::<usize>>::new();
        for (i, dim_index) in range.iter().zip(0..self.dim.len()) {
            let mut pos = i.0;
            let mut all_index = Vec::new();
            while pos < i.1 {
                all_index.push(pos);
                pos += step_dim[dim_index];
            }
            //println!("{:?}", &all_index);
            index.push(all_index);
        }

        let d = self.dim.len();
        let mut pos_index = vec![0; d];
        let mut self_index = vec![0; d];
        loop {
            //println!("pos_index: {:?}", pos_index);
            for i in 0..d {
                self_index[i] = index[i][pos_index[i]];
            }
            //let value = self.get(&self_index);
            //ret.set(&pos_index, value);
            self.set(&self_index, val.get(&pos_index));

            for dim_index in 0..d {
                pos_index[d-1-dim_index] += 1;
                if pos_index[d-1-dim_index] >= val.size()[d-1-dim_index] {
                    pos_index[d-1-dim_index] = 0;
                } else {
                    break;
                }
            }
            
            if pos_index == vec![0; d] {
                break;
            }
        }
    }

    pub fn _iter_patch<F>(&self, dim: Option<&[usize]>, keep_dim: bool, closure: F) -> GenTensor<T>
    where F: Fn(&[T]) -> T {
        // take the whole tensor as the patch.
        if dim.is_none() {
            let ret_dim;
            if keep_dim {
                ret_dim = vec![1; self.size().len()];
            } else {
                ret_dim = vec![1];
            }
            return GenTensor::new_raw(&vec![closure(self.get_data())], &ret_dim)
        }
        let dim = dim.unwrap();

        // build return tensor dimension.
        let mut ret_dim = Vec::new();
        for i in 0..self.size().len() {
            if dim.contains(&i) {
                if keep_dim {
                    ret_dim.push(1);
                }
            } else {
                ret_dim.push(self.size()[i]);
            }
        }
        let mut ret = Self::empty(&ret_dim);

         
        let kept_dim: Vec<usize> = (0..self.size().len()).filter(|x| !dim.contains(&x)).collect();
        let mut index = vec![0; kept_dim.len()];
        loop {
            let mut patch_index: Vec::<(usize, usize)> = Vec::new();
            let mut output_index: Vec<usize> = Vec::new();
            let mut kept_dim_step = 0;
            for i in 0..self.size().len() {
                if dim.contains(&i) {
                    patch_index.push((0, self.size()[i]));
                    if keep_dim {
                        output_index.push(0);
                    }
                } else {
                    patch_index.push((index[kept_dim_step], index[kept_dim_step]+1));
                    output_index.push(index[kept_dim_step]);
                    kept_dim_step += 1;
                }
            }
            //println!("index: {:?}, patch_index: {:?}, output_index: {:?}", index, patch_index, output_index);

            let value = closure(self.get_patch(&patch_index, None).get_data());
            ret.set(&output_index, value);
            
            for i in 0..index.len() {
                index[kept_dim.len() -i -1] += 1;
                if index[kept_dim.len() -i -1] >= self.size()[kept_dim[kept_dim.len() -i -1]] {
                    index[kept_dim.len() -i -1] = 0;
                } else {
                    break
                }
            }

            if index == vec![0; kept_dim.len()] {
                break
            }
        }
        
        ret
    }

    pub fn _dim_statistic<F>(&self, dim: usize, keepdim: bool, closure: F) -> GenTensor<T>
    where F: Fn(usize, usize, usize, usize, usize) -> T {
        if self.dim.len() <= dim {
            panic!("Tensor has dimension {:?}, mean() get dim of {}", self.dim, dim);
        }
        
        let mut ret_dim;
        if keepdim {
            ret_dim = self.dim.to_vec();
            ret_dim[dim] = 1;
        } else {
            ret_dim = Vec::new();
            for (i, index) in self.dim.iter().zip(0..self.dim.len()) {
                if index != dim {
                    ret_dim.push(*i);
                }
            }
        }
        
        let mut cap = 1;
        for i in &ret_dim {
            cap *= i;
        }

        let mut outer_size = 1;
        let mut inner_size = 1;
        for i in 0..self.dim.len() {
            if i < dim {
                outer_size *= self.dim[i];
            }
            if i > dim {
                inner_size *= self.dim[i];
            }
        }
        
        let mut data = Vec::with_capacity(cap);
        let over = self.dim[dim];
        let stride = self.stride();
        let step = stride[dim];

        for k in 0..outer_size {
            for j in 0..inner_size {
                let val = closure(over, k, j, inner_size, step);
                data.push(val);
            }
        }
        
        GenTensor {
            d: data,
            dim: ret_dim,
        }
    }

    // reduction ops


    // Pointwise Ops
    
    fn _pointwise<F>(&self, closure: F) -> GenTensor<T>
    where F: Fn(&T) -> T {
        let mut ret = GenTensor {
            d: Vec::with_capacity(self.d.len()),
            dim: self.dim.clone(),
        };

        for i in &self.d {
            ret.d.push(closure(i));
        }
        ret
    }
    // abs
    pub fn abs(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.abs()
        })
    }
    // acos
    pub fn acos(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.acos()
        })
    }
    // add, there is one.
    // addcdiv
    // addcmul
    // angle
    // asin
    pub fn asin(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.asin()
        })
    }
    // atan
    pub fn atan(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.atan()
        })
    }
    // atan2
    // bitwise_not
    // bitwise_and
    // bitwise_or
    // bitwise_xor
    // ceil
    pub fn ceil(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.ceil()
        })
    }
    // clamp
    pub fn clamp(&self, min: T, max: T) -> GenTensor<T> {
        let mut ret = GenTensor {
            d: Vec::with_capacity(self.d.len()),
            dim: self.dim.clone(),
        };

        for i in &self.d {
            let value;
            if *i < min {
                value = min;
            } else if *i <= max {
                value = *i;
            } else {
                value = max;
            }
            ret.d.push(value);
        }
        ret
    }
    // conj
    // cos
    pub fn cos(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.cos()
        })
    }
    // cosh
    pub fn cosh(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.cosh()
        })
    }
    // div, there is one.
    // digamma
    //pub fn digamma(&self) -> GenTensor<T> {
    //    self._pointwise(|x| {
    //        x.digamma()
    //    })
    //}
    // erf
    // erfc
    // erfinv
    // exp
    pub fn exp(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.exp()
        })
    }
    // expm1
    pub fn expm1(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.exp_m1()
        })
    }
    // floor
    pub fn floor(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.floor()
        })
    }
    // floor_divide
    // fmod
    // frac
    pub fn frac(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.fract()
        })
    }
    // imag
    // lerp, this is on Tensor.
    // lgamma
    // log
    pub fn log(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.ln()
        })
    }
    // log10
    pub fn log10(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.log10()
        })
    }
    // log1p
    pub fn log1p(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.ln_1p()
        })
    }

    /// Better log(1 + exp(x))
    /// see https://cran.r-project.org/web/packages/Rmpfr/vignettes/log1mexp-note.pdf
    pub fn log1pexp(&self) -> GenTensor<T> {
        let mut ret = GenTensor {
            d: Vec::with_capacity(self.d.len()),
            dim: self.dim.to_vec(),
        };
        for i in &self.d {
            if i <= &T::from(-37).expect("") {
                ret.d.push(i.exp());
            } else if i > &T::from(-37).expect("") && i <= &T::from(18).expect("") {
                ret.d.push(i.exp().ln_1p());
            } else if i > &T::from(-18).expect("") && i <= &T::from(33.3).expect("") {
                ret.d.push(*i + i.mul(T::from(-1).expect("")).exp());
            } else {
                ret.d.push(*i);
            }
        }
        ret
    }
    
    // log2
    pub fn log2(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.log2()
        })
    }
    // logical_and
    // logical_not
    // logical_or
    // logical_xor
    // mul, there is one
    // mvlgamma
    // neg
    pub fn neg(&self) -> GenTensor<T> {
        let mut ret = GenTensor {
            d: Vec::with_capacity(self.d.len()),
            dim: self.dim.to_vec(),
        };

        for i in &self.d {
            ret.d.push(i.mul(T::zero() - T::one()));
        }
        ret
    }
    
    // polygamma
    // pow
    pub fn pow(&self, n: T) -> GenTensor<T> {
        self._pointwise(|x| {
            x.powf(n)
        })
    }
    // real
    // reciprocal
    pub fn reciprocal(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.recip()
        })
    }
    // remainder
    // round
    pub fn round(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.round()
        })
    }
    // rsqrt
    pub fn rsqrt(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.sqrt()/(*x)
        })
    }
    
    pub fn sigmoid(&self) -> GenTensor<T> {
        let mut ret = GenTensor {
            d: self.d.to_vec(),
            dim: self.dim.to_vec(),
        };

        for i in 0..self.d.len() {
            if self.d[i] > T::zero() {
                ret.d[i] = T::one()/(T::one() + self.d[i].neg().exp());
            }
            else {
                ret.d[i] = self.d[i].exp()/(T::one() + self.d[i].exp());
            }
        }
        ret
    }

    // sign
    pub fn sign(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            if *x == T::zero() {
                T::zero()
            } else if *x > T::zero() {
                T::one()
            } else {
                T::zero() - T::one()
            }
        })
    }
    // sin
    pub fn sin(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.sin()
        })
    }
    // sinh
    pub fn sinh(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.sinh()
        })
    }
    // sqrt
    pub fn sqrt(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.sqrt()
        })
    }
    // square
    pub fn square(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            (*x)*(*x)
        })
    }
    // tan
    pub fn tan(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.tan()
        })
    }
    // tanh
    pub fn tanh(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.tanh()
        })
    }
    // true_divide
    // trunc
    pub fn trunc(&self) -> GenTensor<T> {
        self._pointwise(|x| {
            x.trunc()
        })
    }
    
    pub fn _right_broadcast<F>(&self, o: &GenTensor<T>, closure: F) -> GenTensor<T>
    where F: Fn(&T, &T) -> T {
        let mut ret = GenTensor {
            d: Vec::with_capacity(self.d.len()),
            dim: self.dim.clone(),
        };
        // with same shape.
        if self.d.len() == o.d.len() {
            for (v1, v2) in self.d.iter().zip(o.d.iter()) {
                ret.d.push(closure(v1, v2));
            }
        // right single scale
        } else if o.dim.len() == 1 && o.dim[0] == 1{
            for i in 0..self.d.len() {
                ret.d.push(closure(&self.d[i], &o.d[0]));
            }
        } else {
            if self.d.len() < o.d.len() {
                panic!("right-hand broadcast only.");
            }
            if self.dim.len() <= o.dim.len() {
                panic!("unmatched dimension. {}, {}", self.dim.len(), o.dim.len());
            }
            for i in 0..o.dim.len() {
                if o.dim[o.dim.len()-i-1] != self.dim[self.dim.len()-i-1] {
                    panic!("unmatched size.");
                }
            }

            // do repeat add
            let mut index = 0;
            for i in 0..self.d.len() {
                ret.d.push(closure(&self.d[i], &o.d[index]));
                index += 1;
                if index >= o.d.len() {
                    index = 0;
                }
            }
        }
        ret
    }
    
    /// element-wise add with right-hand broadcast.
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::new_raw(&vec![1.,2.,3.,4.,], &vec![2,2]);
    /// let m2 = GenTensor::<f64>::new_raw(&vec![1.,2.,3.,4.,], &vec![2,2]);
    /// let m3 = m1.add(&m2);
    /// assert_eq!(m3.get(&vec![0,0]), 2.);
    /// assert_eq!(m3.get(&vec![1,1]), 8.);
    /// ```
    pub fn add(&self, o: &GenTensor<T>) -> GenTensor<T> {
        self._right_broadcast(o, |x, y| *x + *y)
     }
    pub fn sub(&self, o: &GenTensor<T>) -> GenTensor<T> {
        self._right_broadcast(o, |x, y| *x - *y)
    }
    pub fn mul(&self, o: &GenTensor<T>) -> GenTensor<T> {
        self._right_broadcast(o, |x, y| *x * *y)
    }
    pub fn div(&self, o: &GenTensor<T>) -> GenTensor<T> {
        self._right_broadcast(o, |x, y| *x / *y)
    }

    /// matrix multiplication
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::new_raw(&vec![1.,2.,3.,4.,5.,6.], &vec![3,2]);
    /// let m2 = GenTensor::<f64>::new_raw(&vec![2.,3.,4.,5.,6.,7.], &vec![2,3]);
    /// let mut result = m1.mm(&m2);
    /// assert!(result == GenTensor::<f64>::new_raw(&vec![12.,15.,18.,26.,33.,40.,40.,51.,62.,], &vec![3,3]), "");
    /// ```
    pub fn mm(&self, o: &GenTensor<T>) -> GenTensor<T>{
        if self.dim.len() != 2 || o.dim.len() != 2 {
            panic!("Not a matrix input.");
        }
        let ls = self.dim[0];
        let rs = o.dim[1];
        let mut ret = GenTensor {
            d: Vec::with_capacity(ls*rs),
            dim: vec![ls, rs],
        };
        let lstride = self.stride();
        let rstride = o.stride();
        for i in 0..ls {
            for j in 0..rs {
                let mut tsum = T::zero();
                for k in 0..self.dim[1] {
                    tsum = tsum
                        + self.d[i*lstride[0] + k] * o.d[k*rstride[0] + j];
                }
                ret.d.push(tsum);
            }
        }
        ret
    }

    /// matrix multiplication of two tensor
    pub fn matmul(&self, o: &GenTensor<T>) -> GenTensor<T> {
        if self.dim[self.dim.len()-1] != o.dim[0] {
            panic!("matmul expect matched size {:?}, {:?}", self.dim, o.dim);
        }
        let inner = o.dim[0];
        let mut cap = 1;
        let mut odim = Vec::new();
        let mut lloop = 1;
        let mut rloop = 1;
        for i in 0..self.dim.len()-1 {
            cap *= self.dim[i];
            odim.push(self.dim[i]);
            lloop *= self.dim[i];
        }
        for i in 1..o.dim.len() {
            cap *= o.dim[i];
            odim.push(o.dim[i]);
            rloop *= o.dim[i];
        }

        let mut ret = GenTensor {
            d: Vec::with_capacity(cap),
            dim: odim,
        };
        
        let lstride = self.stride();
        let rstride = o.stride();
        for i in 0..lloop {
            for j in 0..rloop {
                let mut tsum = T::zero();
                for k in 0..inner {
                    tsum = tsum
                        + self.d[i*lstride[0] + k] * o.d[k*rstride[0] + j];
                }
                ret.d.push(tsum);
            }
        }
        ret
    }

    /// outer product of right-most dimensions.
    pub fn outer(&self, o: &GenTensor<T>, avg: Option<bool>) -> GenTensor<T> {
        let mut dim = Vec::new();
        let mut data;
        let mut cap = 1;
        let mut outer_size = 1;
        let left_dim;
        let right_dim;
        if self.dim.len() == o.dim.len()
            && self.dim[0..self.dim.len()-1] == o.dim[0..self.dim.len()-1] {
                left_dim = self.dim[self.dim.len()-1];
                right_dim = o.dim[self.dim.len()-1];
                for i in 0..self.dim.len()-1 {
                    dim.push(self.dim[i]);
                    cap *= self.dim[i];
                    outer_size *= self.dim[i];
                }
                dim.push(left_dim);
                cap *= left_dim;
                dim.push(right_dim);
                cap *= right_dim;
                if avg.is_some() && avg.unwrap() {
                    data = vec![T::zero(); left_dim*right_dim];
                    dim = vec![left_dim, right_dim];
                } else {
                    data = Vec::with_capacity(cap);
                }
            } else {
                panic!("bad size for outer: {:?}, {:?}", self.dim, o.dim);
            }

        
        if avg.is_some() && avg.unwrap() {
            for k in 0..outer_size {
                let mut new_data = Vec::with_capacity(left_dim*right_dim);
                for i in 0..left_dim {
                    for j in 0..right_dim {
                        new_data.push(self.d[i + k*left_dim] * o.d[j + k*right_dim]);
                    }
                }
                for i in 0..new_data.len() {
                    data[i] = data[i] + new_data[i];
                }
            }
            for i in 0..data.len() {
                data[i] = data[i] / T::from(outer_size).expect("");
            }
            GenTensor {
                d: data,
                dim: dim,
            }
        } else {
            for k in 0..outer_size {
                for i in 0..left_dim {
                    for j in 0..right_dim {
                        data.push(self.d[i + k*left_dim] * o.d[j + k*right_dim]);
                    }
                }
            }
            GenTensor {
                d: data,
                dim: dim,
            }
        }
        
        
        
    }

    pub fn squared_error(t1: &Self, t2: &Self) -> GenTensor<T> {
        let mut ret = GenTensor {
            d: Vec::with_capacity(t1.d.len()),
            dim: t1.dim.to_vec(),
        };
        for (v1, v2) in t1.d.iter().zip(t2.d.iter()) {
            ret.d.push((*v1 - *v2)*(*v1 - *v2));
        }
        ret
    }


    // Comparison Ops


    pub fn all_close(&self, o: &GenTensor<T>) -> GenTensor<T> {
        self.eq_t(o)
    }

    pub fn arg_sort(&self, dim: usize, descending: bool) -> GenTensor<T> {
        let mut d = self.d.to_vec();

        let mut outer_size = 1;
        let mut inner_size = 1;

        for (i, index) in self.dim.iter().zip(0..self.dim.len()) {
            if index < dim {
                outer_size *= i;
            } else if index > dim {
                inner_size *= i;
            }
        }

        let stride = self.stride()[dim];
        let size = self.dim[dim];

        for i in 0..outer_size {
            for j in 0..inner_size {
                let mut collected = Vec::<(T, usize)>::with_capacity(size);
                for k in 0..size {
                    collected.push((self.d[k*stride + j + i*inner_size*size], k));
                }
                collected.sort_unstable_by(|a, b| {
                    let porder = a.0.partial_cmp(&b.0).unwrap();
                    if descending {
                        porder
                    } else {
                        porder.reverse()
                    }
                });
                let (_left, right): (Vec<_>, Vec<_>) = collected.iter().cloned().unzip();
                for k in 0..size {
                    d[k*stride + j + i*inner_size*size] = T::from(right[k]).expect("");
                }
            }
        }

        GenTensor {
            d: d,
            dim: self.dim.to_vec()
        }
    }

    /// Computes element-wise equality
    /// use eq_t instead, as eq is reserved for == overloading.
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::new_raw(&vec![1.,2.,3.,4.,5.,6.], &vec![3,2]);
    /// let m2 = GenTensor::<f64>::new_raw(&vec![1.,2.,3.,4.,5.,6.], &vec![3,2]);
    /// assert_eq!(m1.eq_t(&m2).get(&vec![0,0]), 1.);
    /// assert_eq!(m1.eq_t(&m2).get(&vec![2,1]), 1.);
    /// ```
    pub fn eq_t(&self, o: &GenTensor<T>) -> GenTensor<T> {
        let mut cmp = Vec::<T>::with_capacity(self.d.len());
        for (v1, v2) in self.d.iter().zip(o.d.iter()) {
            if (*v1-*v2).abs() < T::min_positive_value().sqrt() {
                cmp.push(T::one(),);
            } else {
                cmp.push(T::zero());
            }
        }
        GenTensor {
            d: cmp,
            dim: self.dim.to_vec(),
        }
    }

    /// true if two tensors have the same size and elements, false otherwise.
    ///
    /// ```
    /// # use tensor_rs::tensor::gen_tensor::*;
    /// let m1 = GenTensor::<f64>::fill(1., &vec![3,5,2]);
    /// let m2 = GenTensor::<f64>::fill(1., &vec![3,5,2]);
    /// assert_eq!(m1.equal(&m2), true)
    /// ```
    pub fn equal(&self, o: &GenTensor<T>) -> bool {
        if self.dim.len() != o.dim.len() || self.dim != o.dim {
            return false;
        }

        if self.d.len() != o.d.len() {
            return false;
        }
        
        let mut same = true;
        for (v1, v2) in self.d.iter().zip(o.d.iter()) {
            if (*v1-*v2).abs() > T::min_positive_value().sqrt() {
                same = false;
                break;
            }
        }
        same
    }

    pub fn ge(&self, o: &GenTensor<T>) -> GenTensor<T> {
        if self.size() != o.size() {
            panic!("max needs two tensor have the same size, {:?}, {:?}", self.dim, o.dim);
        }
        let mut ret = GenTensor::empty(&self.dim);

        for ((a, b), c) in self.d.iter().zip(o.d.iter()).zip(ret.d.iter_mut()) {
            if a >= b {
                *c = T::one();
            } else {
                *c = T::zero();
            }
        }
        ret
    }

    pub fn gt(&self, o: &GenTensor<T>) -> GenTensor<T> {
        if self.size() != o.size() {
            panic!("max needs two tensor have the same size, {:?}, {:?}", self.dim, o.dim);
        }
        let mut ret = GenTensor::empty(&self.dim);

        for ((a, b), c) in self.d.iter().zip(o.d.iter()).zip(ret.d.iter_mut()) {
            if a > b {
                *c = T::one();
            } else {
                *c = T::zero();
            }
        }
        ret
    }

    //pub fn isfinite(&self, o: &GenTensor<T>) -> GenTensor<T> {
    //    GenTensor::new()
    //}
    //
    //pub fn isinf(&self, o: &GenTensor<T>) -> GenTensor<T> {
    //    GenTensor::new()
    //}
    //
    //pub fn isnan(&self, o: &GenTensor<T>) -> GenTensor<T> {
    //    GenTensor::new()
    //}
    //
    //pub fn kthvalue(&self, o: &GenTensor<T>) -> GenTensor<T> {
    //    GenTensor::new()
    //}
    // le
    pub fn le(&self, o: &GenTensor<T>) -> GenTensor<T> {
        if self.size() != o.size() {
            panic!("max needs two tensor have the same size, {:?}, {:?}", self.dim, o.dim);
        }
        let mut ret = GenTensor::empty(&self.dim);

        for ((a, b), c) in self.d.iter().zip(o.d.iter()).zip(ret.d.iter_mut()) {
            if a <= b {
                *c = T::one();
            } else {
                *c = T::zero();
            }
        }
        ret
    }
    // lt
    pub fn lt(&self, o: &GenTensor<T>) -> GenTensor<T> {
        if self.size() != o.size() {
            panic!("max needs two tensor have the same size, {:?}, {:?}", self.dim, o.dim);
        }
        let mut ret = GenTensor::empty(&self.dim);

        for ((a, b), c) in self.d.iter().zip(o.d.iter()).zip(ret.d.iter_mut()) {
            if a < b {
                *c = T::one();
            } else {
                *c = T::zero();
            }
        }
        ret
    }
    // max, 
    //pub fn max(&self, o: Option<&GenTensor<T>>, dim: Option<usize>, keep_dim: Option<bool>) -> GenTensor<T> {
    //    if o.is_none() && dim.is_none() && keep_dim.is_none() {
    //        max_all()
    //    } else if o.is_some() && dim.is_none() && keep_dim.is_none() {
    //        max_pair()
    //    }
    //}

    // ne
    pub fn ne(&self, o: &GenTensor<T>) -> GenTensor<T> {
        if self.size() != o.size() {
            panic!("max needs two tensor have the same size, {:?}, {:?}", self.dim, o.dim);
        }
        

        let data = self.d.iter().zip(
            o.d.iter())
            .map(|(x, y)|
                 if *x != *y {
                     T::one()
                 } else {
                     T::zero()
                 }
        ).collect();
        GenTensor {
            d: data,
            dim: self.dim.to_vec(),
        }
    }
    // sort
    // topk
    
}


/// ```
/// # use tensor_rs::tensor::gen_tensor::*;
/// let m1 = GenTensor::<f64>::fill(1., &vec![3,5,2]);
/// let m2 = GenTensor::<f64>::fill(1., &vec![3,5,2]);
/// assert_eq!(m1==m2, true)
/// ```
impl<T> PartialEq for GenTensor<T> where T: num_traits::Float {
    fn eq(&self, other: &Self) -> bool {
        if self.equal(other) {
            true
        } else {
            false
        }
    }
}
impl<T> Eq for GenTensor<T> where T: num_traits::Float {}

impl fmt::Display for GenTensor<f32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dim.len() == 2 {
            write!(f, "[")?;
            for i in 0..self.dim[0] {
                write!(f, "[")?;
                for j in 0..self.dim[1] {
                    write!(f, "{}, ", self.get(&vec![i, j]))?;
                }
                write!(f, "]\n")?;
            }
            write!(f, "]\n")
        } else {
            write!(f, "{:?}\n", self.dim)?;
            write!(f, "{:?}", self.d)            
        }
    }
}
impl fmt::Display for GenTensor<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.dim)?;
        write!(f, "{:?}\n", self.d)
    }
}

impl fmt::Debug for GenTensor<f32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if self.dim.len() == 2 {
            write!(f, "[")?;
            for i in 0..self.dim[0] {
                write!(f, "[")?;
                for j in 0..self.dim[1] {
                    write!(f, "{}, ", self.get(&vec![i, j]))?;
                }
                write!(f, "]\n")?;
            }
            write!(f, "]\n")
        } else {
            write!(f, "{:?}\n", self.dim)?;
            write!(f, "{:?}", self.d)            
        }
    }
}
impl fmt::Debug for GenTensor<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dim.len() == 2 {
            write!(f, "[")?;
            for i in 0..self.dim[0] {
                write!(f, "[")?;
                for j in 0..self.dim[1] {
                    write!(f, "{}, ", self.get(&vec![i, j]))?;
                }
                write!(f, "]\n")?;
            }
            write!(f, "]\n")
        } else {
            write!(f, "{:?}\n", self.dim)?;
            write!(f, "{:?}", self.d)            
        }
    }
}

impl<T> Clone for GenTensor<T> where T: num_traits::Float {
    fn clone(&self) -> Self {
        GenTensor {
            d: self.d.to_vec(),
            dim: self.dim.to_vec(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index2dimpos() {
        let a = GenTensor::<f32>::empty(&vec![10, 5, 3, 4]);

        let b = a.index2dimpos(10);
        assert_eq!(b, vec![0, 0, 2, 2]);

    }

    #[test]
    fn test_gentensor() {
        {
            let mut m2 = GenTensor::<f64>::new_raw(&vec![1., 2., 3., 4.,], &vec![2, 2]);
            *m2.get_mut(&vec![0,0]) = 5.;
            assert_eq!(m2.get_raw(), vec![5., 2., 3., 4.,])
        }
    }

    #[test]
    fn test_gen_tensor_get() {
        {
            let m1 = GenTensor::<f64>::fill(1., &vec![10, 3, 28, 30]);
            assert_eq!(m1.get_n().get_raw(), vec![10.]);
            assert_eq!(m1.get_c().get_raw(), vec![3.]);
            assert_eq!(m1.get_h().get_raw(), vec![28.]);
            assert_eq!(m1.get_w().get_raw(), vec![30.]);

            let result = std::panic::catch_unwind(
                ||
                    m1.get_d().get_raw()
            );
            assert!(result.is_err());
        }
    }

    #[test]
    fn outer() {
        let a = GenTensor::<f32>::fill(1., &vec![10, 2]);
        let b = GenTensor::<f32>::fill(1., &vec![10, 3]);
        let c = a.outer(&b, None);
        assert_eq!(*c.size(), vec![10, 2, 3]);
        //println!("{}", c);
        let d = b.outer(&a, None);
        assert_eq!(*d.size(), vec![10, 3, 2]);

        let e = a.outer(&b, Some(true));
        assert_eq!(e, GenTensor::ones(&[2, 3]));
    }

    #[test]
    fn get_patch() {
        let a = GenTensor::new_raw(&GenTensor::<f32>::arange(30).get_data(), &[2, 3, 5]);
        let b = a.get_patch(&vec![(0, 2), (0, 2), (2, 3)][..], Option::None);
        assert_eq!(b, GenTensor::<f32>::new_raw(&vec![2.0, 7.0, 17.0, 22.0][..], &vec![2, 2, 1][..]));
    }

    #[test]
    fn set_patch() {
        let mut a = GenTensor::new_raw(&GenTensor::<f32>::arange(30).get_data(), &[2, 3, 5]);
        let b = GenTensor::<f32>::ones(&[1, 3, 5]);
        a.set_patch(&b, &[(1,2), (0,3), (0,5)], None);
        println!("{:?}", a);
        assert_eq!(a, GenTensor::new_raw(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0], &[2, 3, 5]));
    }

    // Pointwise Ops
    #[test]
    fn ceil() {
        let a = GenTensor::<f32>::new_raw(&vec![0.9213,  1.0887, -0.8858, -1.7683],
                                              &vec![4]);
        
        let ret = a.ceil();

        let expect = GenTensor::<f32>::new_raw(&vec![1., 2., 0., -1.], 
                                               &vec![4]);
        assert_eq!(ret, expect);
    }
    
    #[test]
    fn log1pexp() {
        let a = GenTensor::<f32>::new_raw(&vec![0.9213,  1.0887, -0.8858, -1.7683],
                                              &vec![4]);
        
        let ret = a.log1pexp();

        let expect = GenTensor::<f32>::new_raw(&vec![1.2563436, 1.3788694, 0.34527916, 0.15753591], 
                                               &vec![4]);
        assert_eq!(ret, expect);
    }
    
    #[test]
    fn sigmoid() {
        let a = GenTensor::<f32>::new_raw(&vec![0.9213,  1.0887, -0.8858, -1.7683],
                                              &vec![4]);
        
        let ret = a.sigmoid();

        let expect = GenTensor::<f32>::new_raw(&vec![0.71530694, 0.7481369, 0.29197732, 0.14575386], 
                                               &vec![4]);
        assert_eq!(ret, expect);
    }

    #[test]
    fn sign() {
        let a = GenTensor::<f32>::new_raw(&vec![0.9213,  0.0, -0.0, -1.7683],
                                              &vec![4]);
        
        let ret = a.sign();

        let expect = GenTensor::<f32>::new_raw(&vec![1.0, 0.0, 0.0, -1.0],
                                               &vec![4]);
        assert_eq!(ret, expect);
    }
    

    // Comparison Ops
    #[test]
    fn arg_sort() {
        let a = GenTensor::<f32>::new_raw(&vec![0.0785,  1.5267, -0.8521,  0.4065,
                                                    0.1598,  0.0788, -0.0745, -1.2700,
                                                    1.2208,  1.0722, -0.7064,  1.2564,
                                                    0.0669, -0.2318, -0.8229, -0.9280,],
                                              &vec![4, 4]);
        
        let index = a.arg_sort(1, true);

        let expect = GenTensor::<f32>::new_raw(&vec![2., 0., 3., 1., 
                                                     3., 2., 1., 0., 
                                                     2., 1., 0., 3., 
                                                     3., 2., 1., 0.], 
                                               &vec![4, 4]);
        assert_eq!(index, expect);
    }

    #[test]
    fn ne() {
        let a = GenTensor::<f32>::new_raw(&vec![1., 3., 10., 11.], &vec![2,2]);
        let b = GenTensor::<f32>::new_raw(&vec![2., 3., 10., 6.], &vec![2,2]);
        let c = a.ne(&b);
        assert_eq!(c, GenTensor::<f32>::new_raw(&vec![1., 0., 0., 1.], &vec![2,2]));
    }
    

}
