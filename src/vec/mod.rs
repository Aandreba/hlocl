#[cfg(test)]
extern crate std;

flat_mod!(base);

impl_prog! {
    pub VectorManager = {
        //pub xscal as XScalProgram @ "kernels/xscal.ocl" => fscal_add, fscal_sub, fscal_mul, fscal_div, fscal_sub_inv, fscal_div_inv, fscal_add_assign, fscal_sub_assign, fscal_mul_assign, fscal_div_assign;
        pub xarith as XArithProgram @ "kernels/xarith.ocl" => add, sub, mul, div, add_assign, sub_assign, mul_assign, div_assign
        //pub xvert as XVertProgram @ "kernels/xvert.ocl" => abs_reg, sqrt_reg, abs_assign, sqrt_assign;
        //pub xhoz as XHozProgram @ "kernels/xhoz.ocl" => sum, prod, dot_reg, sum_epilogue, prod_epilogue
    }
}