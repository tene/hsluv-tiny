#![no_std]
use core::f32::consts::PI;
use core::str;
use heapless::{consts::*, String};
#[allow(unused_imports)]
use num_traits::float::Float;

const M: [[f32; 3]; 3] = [
    [3.240969941904521, -1.537383177570093, -0.498610760293],
    [-0.96924363628087, 1.87596750150772, 0.041555057407175],
    [0.055630079696993, -0.20397695888897, 1.056971514242878],
];

const M_INV: [[f32; 3]; 3] = [
    [0.41239079926595, 0.35758433938387, 0.18048078840183],
    [0.21263900587151, 0.71516867876775, 0.072192315360733],
    [0.019330818715591, 0.11919477979462, 0.95053215224966],
];

const REF_Y: f32 = 1.0;
const REF_U: f32 = 0.19783000664283;
const REF_V: f32 = 0.46831999493879;
// CIE LUV constants
const KAPPA: f32 = 903.2962962;
const EPSILON: f32 = 0.0088564516;

#[derive(Copy, Clone, PartialEq)]
pub struct Hsluv {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl Hsluv {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self { h, s, l }
    }
    pub fn to_rgb(self) -> Rgb {
        self.to_lch().to_luv().to_xyz().to_rgb()
    }
    pub fn to_lch(self) -> Lch {
        match self.l {
            l if l > 99.9999999 => Lch::new(100.0, 0.0, self.h),
            l if l < 0.00000001 => Lch::new(0.0, 0.0, self.h),
            _ => {
                let mx = max_chroma_for(self.l, self.h);
                let c = mx / 100.0 * self.s;
                Lch::new(self.l, c, self.h)
            }
        }
    }
}

impl Into<(f32, f32, f32)> for Hsluv {
    fn into(self) -> (f32, f32, f32) {
        (self.h, self.s, self.l)
    }
}

impl From<(f32, f32, f32)> for Hsluv {
    fn from(triple: (f32, f32, f32)) -> Self {
        Self::new(triple.0, triple.1, triple.2)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Lch {
    pub l: f32,
    pub c: f32,
    pub h: f32,
}

impl Lch {
    pub fn new(l: f32, c: f32, h: f32) -> Self {
        Self { l, c, h }
    }
    /// Convert LCH to LUV
    pub fn to_luv(self) -> Luv {
        let hrad = degrees_to_radians(self.h);
        let u = hrad.cos() * self.c;
        let v = hrad.sin() * self.c;

        Luv::new(self.l, u, v)
    }
}

impl Into<(f32, f32, f32)> for Lch {
    fn into(self) -> (f32, f32, f32) {
        (self.l, self.c, self.h)
    }
}

impl From<(f32, f32, f32)> for Lch {
    fn from(triple: (f32, f32, f32)) -> Self {
        Self::new(triple.0, triple.1, triple.2)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Luv {
    pub l: f32,
    pub u: f32,
    pub v: f32,
}

impl Luv {
    pub fn new(l: f32, u: f32, v: f32) -> Self {
        Self { l, u, v }
    }
    pub fn to_xyz(self) -> Xyz {
        let Self { l, u, v } = self;
        if self.l == 0.0 {
            return Xyz::new(0.0, 0.0, 0.0);
        }

        let var_y = f_inv(l);
        let var_u = u / (13.0 * l) + REF_U;
        let var_v = v / (13.0 * l) + REF_V;

        let y = var_y * REF_Y;
        let x = 0.0 - (9.0 * y * var_u) / ((var_u - 4.0) * var_v - var_u * var_v);
        let z = (9.0 * y - (15.0 * var_v * y) - (var_v * x)) / (3.0 * var_v);

        Xyz::new(x, y, z)
    }
}

impl Into<(f32, f32, f32)> for Luv {
    fn into(self) -> (f32, f32, f32) {
        (self.l, self.u, self.v)
    }
}

impl From<(f32, f32, f32)> for Luv {
    fn from(triple: (f32, f32, f32)) -> Self {
        Self::new(triple.0, triple.1, triple.2)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Xyz {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Xyz {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn to_rgb(self) -> Rgb {
        let xyz = [self.x, self.y, self.z];
        let r = from_linear(dot_product(&M[0], &xyz));
        let g = from_linear(dot_product(&M[1], &xyz));
        let b = from_linear(dot_product(&M[2], &xyz));
        Rgb::new(r, g, b)
    }
}
impl Into<(f32, f32, f32)> for Xyz {
    fn into(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}

impl From<(f32, f32, f32)> for Xyz {
    fn from(triple: (f32, f32, f32)) -> Self {
        Self::new(triple.0, triple.1, triple.2)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Rgb {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }
}

impl Into<(f32, f32, f32)> for Rgb {
    fn into(self) -> (f32, f32, f32) {
        (self.r, self.g, self.b)
    }
}

impl From<(f32, f32, f32)> for Rgb {
    fn from(triple: (f32, f32, f32)) -> Self {
        Self::new(triple.0, triple.1, triple.2)
    }
}

/// Convert HSLUV to HEX
pub fn hsluv_to_hex(hsl: (f32, f32, f32)) -> String<U7> {
    rgb_to_hex(hsluv_to_rgb(hsl))
}

/// Convert HPLUV to HEX
pub fn hpluv_to_hex(hsl: (f32, f32, f32)) -> String<U7> {
    rgb_to_hex(hpluv_to_rgb(hsl))
}

/// Convert HEX to HSLUV
pub fn hex_to_hsluv(hex: &str) -> (f32, f32, f32) {
    rgb_to_hsluv(hex_to_rgb(hex))
}

/// Convert HEX to HPLUV
pub fn hex_to_hpluv(hex: &str) -> (f32, f32, f32) {
    rgb_to_hpluv(hex_to_rgb(hex))
}

/// Convert HPLUV to LCH
pub fn hpluv_to_lch(hpl: (f32, f32, f32)) -> (f32, f32, f32) {
    let (h, p, l) = hpl;
    match l {
        l if l > 99.9999999 => (100.0, 0.0, h),
        l if l < 0.00000001 => (0.0, 0.0, h),
        _ => {
            let mx = max_safe_chroma_for(l);
            let c = mx / 100.0 * p;
            (l, c, h)
        }
    }
}

/// Convert LCH to HSLUV
pub fn lch_to_hsluv(lch: (f32, f32, f32)) -> (f32, f32, f32) {
    let (l, c, h) = lch;
    match l {
        l if l > 99.99 => (h, 0.0, 100.0),
        l if l < 0.001 => (h, 0.0, 0.0),
        _ => {
            let mx = max_chroma_for(l, h);
            let s = c / mx * 100.0;
            (h, s, l)
        }
    }
}

/// Convert LCH to HPLUV
pub fn lch_to_hpluv(lch: (f32, f32, f32)) -> (f32, f32, f32) {
    let (l, c, h) = lch;
    match l {
        l if l > 99.99 => (h, 0.0, 100.0),
        l if l < 0.001 => (h, 0.0, 0.0),
        _ => {
            let mx = max_safe_chroma_for(l);
            let s = c / mx * 100.0;
            (h, s, l)
        }
    }
}

pub fn hsluv_to_lch(hsl: (f32, f32, f32)) -> (f32, f32, f32) {
    let hsl: Hsluv = hsl.into();
    let lch = hsl.to_lch();
    lch.into()
}

pub fn lch_to_luv(lch: (f32, f32, f32)) -> (f32, f32, f32) {
    let lch: Lch = lch.into();
    let luv = lch.to_luv();
    luv.into()
}

/// Convert LCH to RGB
pub fn lch_to_rgb(lch: (f32, f32, f32)) -> (f32, f32, f32) {
    xyz_to_rgb(luv_to_xyz(lch_to_luv(lch)))
}

/// Convert HSLUV to RGB
pub fn hsluv_to_rgb(hsl: (f32, f32, f32)) -> (f32, f32, f32) {
    xyz_to_rgb(luv_to_xyz(lch_to_luv(hsluv_to_lch(hsl))))
}

/// Convert HPLUV to RGB
pub fn hpluv_to_rgb(hsl: (f32, f32, f32)) -> (f32, f32, f32) {
    lch_to_rgb(hpluv_to_lch(hsl))
}

/// Convert XYZ to RGB
pub fn xyz_to_rgb(xyz: (f32, f32, f32)) -> (f32, f32, f32) {
    let xyz: Xyz = xyz.into();
    let rgb = xyz.to_rgb();
    rgb.into()
}

/// Convert LUV to XYZ
pub fn luv_to_xyz(luv: (f32, f32, f32)) -> (f32, f32, f32) {
    let luv: Luv = luv.into();
    let xyz = luv.to_xyz();
    xyz.into()
}

/// Convert XYZ to LUV
pub fn xyz_to_luv(xyz: (f32, f32, f32)) -> (f32, f32, f32) {
    let (x, y, z) = xyz;
    let l = f(y);

    if l == 0.0 || (xyz == (0.0, 0.0, 0.0)) {
        return (0.0, 0.0, 0.0);
    }

    let var_u = (4.0 * x) / (x + (15.0 * y) + (3.0 * z));
    let var_v = (9.0 * y) / (x + (15.0 * y) + (3.0 * z));
    let u = 13.0 * l * (var_u - REF_U);
    let v = 13.0 * l * (var_v - REF_V);

    (l, u, v)
}

/// Convert RGB to HSLUV
pub fn rgb_to_hsluv(rgb: (f32, f32, f32)) -> (f32, f32, f32) {
    lch_to_hsluv(rgb_to_lch(rgb))
}

/// Convert RGB to HPLUV
pub fn rgb_to_hpluv(rgb: (f32, f32, f32)) -> (f32, f32, f32) {
    lch_to_hpluv(rgb_to_lch(rgb))
}

/// Convert RGB to LCH
pub fn rgb_to_lch(rgb: (f32, f32, f32)) -> (f32, f32, f32) {
    luv_to_lch(xyz_to_luv(rgb_to_xyz(rgb)))
}

/// Convert RGB to XYZ
pub fn rgb_to_xyz(rgb: (f32, f32, f32)) -> (f32, f32, f32) {
    let rgbl = [to_linear(rgb.0), to_linear(rgb.1), to_linear(rgb.2)];
    let x = dot_product(&M_INV[0], &rgbl);
    let y = dot_product(&M_INV[1], &rgbl);
    let z = dot_product(&M_INV[2], &rgbl);
    (x, y, z)
}

/// Convert LUV to LCH
pub fn luv_to_lch(luv: (f32, f32, f32)) -> (f32, f32, f32) {
    let (l, u, v) = luv;
    let c = (u * u + v * v).sqrt();
    if c < 0.001 {
        (l, c, 0.0)
    } else {
        let hrad = f32::atan2(v, u);
        let mut h = radians_to_degrees(hrad);
        if h < 0.0 {
            h += 360.0;
        }
        (l, c, h)
    }
}

/// Convert RGB to HEX
pub fn rgb_to_hex(rgb: (f32, f32, f32)) -> String<U7> {
    use core::fmt::Write;
    let (r, g, b) = rgb_prepare(rgb);
    let mut rv = String::new();
    let _ = write!(rv, "#{:02x}{:02x}{:02x}", r, g, b);
    rv
}

/// Convert HEX to RGB
pub fn hex_to_rgb(raw_hex: &str) -> (f32, f32, f32) {
    let hex = raw_hex.trim_start_matches('#');
    if hex.len() != 6 {
        //panic!("Not a hex string!");
        return (0.0, 0.0, 0.0);
    }
    let mut chunks = hex.as_bytes().chunks(2);
    let red = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
    let green = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
    let blue = i64::from_str_radix(str::from_utf8(chunks.next().unwrap()).unwrap(), 16);
    (
        (red.unwrap_or(0) as f32) / 255.0,
        (green.unwrap_or(0) as f32) / 255.0,
        (blue.unwrap_or(0) as f32) / 255.0,
    )
}

fn f_inv(t: f32) -> f32 {
    if t > 8.0 {
        REF_Y * ((t + 16.0) / 116.0).powf(3.0)
    } else {
        REF_Y * t / KAPPA
    }
}

fn to_linear(c: f32) -> f32 {
    if c > 0.04045 {
        ((c + 0.055) / 1.055).powf(2.4)
    } else {
        c / 12.92
    }
}

fn from_linear(c: f32) -> f32 {
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * (c.powf(1.0 / 2.4)) - 0.055
    }
}

fn f(t: f32) -> f32 {
    if t > EPSILON {
        116.0 * ((t / REF_Y).powf(1.0 / 3.0)) - 16.0
    } else {
        t / REF_Y * KAPPA
    }
}

fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(i, j)| i * j).sum()
}

fn rgb_prepare(rgb: (f32, f32, f32)) -> (u8, u8, u8) {
    (clamp(rgb.0), clamp(rgb.1), clamp(rgb.2))
}

fn clamp(v: f32) -> u8 {
    let mut rounded = (v * 1000.0).round() / 1000.0;
    if rounded < 0.0 {
        rounded = 0.0;
    }
    if rounded > 1.0 {
        rounded = 1.0;
    }
    (rounded * 255.0).round() as u8
}

fn max_chroma_for(l: f32, h: f32) -> f32 {
    let hrad = h / 360.0 * PI * 2.0;

    get_bounds(l)
        .iter()
        .map(|line| length_of_ray_until_intersect(hrad, line))
        .filter(|length| length > &0.0)
        .fold(f32::MAX, f32::min)
}

fn max_safe_chroma_for(l: f32) -> f32 {
    get_bounds(l)
        .iter()
        .map(|line| {
            let x = intersect_line_line((line.0, line.1), (-1.0 / line.0, 0.0));
            distance_from_pole((x, line.1 + x * line.0))
        })
        .fold(f32::MAX, f32::min)
}

fn intersect_line_line(line1: (f32, f32), line2: (f32, f32)) -> f32 {
    (line1.1 - line2.1) / (line2.0 - line1.0)
}

fn distance_from_pole(point: (f32, f32)) -> f32 {
    (point.0.powi(2) + point.1.powi(2)).sqrt()
}

fn get_bounds(l: f32) -> [(f32, f32); 6] {
    let sub1 = ((l + 16.0).powi(3)) / 1560896.0;
    let sub2 = match sub1 {
        s if s > EPSILON => s,
        _ => l / KAPPA,
    };

    let mut bounds = [(0.0, 0.0); 6];

    let mut idx = 0;
    for ms in &M {
        let (m1, m2, m3) = (ms[0], ms[1], ms[2]);
        for t in 0..2i16 {
            let top1 = (284517.0 * m1 - 94839.0 * m3) * sub2;
            let top2 = (838422.0 * m3 + 769860.0 * m2 + 731718.0 * m1) * l * sub2
                - 769860.0 * f32::from(t) * l;
            let bottom = (632260.0 * m3 - 126452.0 * m2) * sub2 + 126452.0 * f32::from(t);

            bounds[idx] = (top1 / bottom, top2 / bottom);
            idx += 1;
        }
    }
    bounds
}

fn length_of_ray_until_intersect(theta: f32, line: &(f32, f32)) -> f32 {
    let (m1, b1) = *line;
    let length = b1 / (theta.sin() - m1 * theta.cos());
    if length < 0.0 {
        -0.0001
    } else {
        length
    }
}

fn radians_to_degrees(rad: f32) -> f32 {
    rad * 180.0 / PI
}

fn degrees_to_radians(deg: f32) -> f32 {
    deg * PI / 180.0
}
