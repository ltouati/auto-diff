use crate::tensor::gen_tensor::GenTensor;

pub trait ElemwiseTensorOp {
    type TensorType;
    type ElementType;

    fn lu(&self) -> Option<[Self::TensorType; 2]>;
    fn det(&self) -> Option<Self::ElementType>;
}


impl<T> ElemwiseTensorOp for GenTensor<T>
where T: num_traits::Float {
    type TensorType = GenTensor<T>;
    type ElementType = T;

    fn lu(&self) -> Option<[Self::TensorType; 2]> {
        // lu is for square matrix only.
        // TODO; handle the batched/3d case.
        if self.size().len() != 2 {
            return None;
        }
        if self.size()[0] != self.size()[1] {
            return None;
        }
        let nr = self.size()[0];
        let mut l = GenTensor::<T>::eye(nr, nr);
        let mut u = self.clone();
        for i in 0..nr-1 {
            let leading = u.get(&[i, i]);
            for j in i+1..nr {
                let multiplier = u.get(&[j, i])/leading;
                l.set(&[j, i], multiplier);
                for k in i..nr {
                    u.set(&[j, k], u.get(&[j, k]) - u.get(&[i, k])*multiplier);
                }
            }
        }

        Some([l, u])
    }
    
    fn det(&self) -> Option<Self::ElementType> {
        if let Some(v) = self.lu() {
            let [l, u] = v;

            Some(T::zero())
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lu() {
        let m = GenTensor::<f64>::new_raw(&[1., 1., 1., 4., 3., -1., 3., 5., 3.], &[3,3]);
        let [l, u] = m.lu().unwrap();
        let el = GenTensor::<f64>::new_raw(&[1., 0., 0., 4., 1., 0., 3., -2., 1.], &[3,3]);
        let eu = GenTensor::<f64>::new_raw(&[1., 1., 1., 0., -1., -5., 0., 0., -10.], &[3,3]);
        assert_eq!(l, el);
        assert_eq!(u, eu);
    }
}
