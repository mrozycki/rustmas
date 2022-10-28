pub fn to_polar((x, y, z): &(f64, f64, f64)) -> (f64, f64, f64) {
    ((x.powi(2) + z.powi(2)).sqrt(), (x / z).atan(), *y)
}
