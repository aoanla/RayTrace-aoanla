use std::ops;
use std::fs::File;
use std::path::Path;
use std::io::BufWriter;
extern crate png;
extern crate ndarray;
extern crate itertools;
use ndarray::Array;
use ndarray::Ix3;
use ndarray::s;
use itertools::Itertools;


#[derive(Copy, Clone)]
pub struct ThreeVec(f64,f64,f64) ;

impl ops::Mul<ThreeVec> for ThreeVec {
        
    type Output = f64;

    fn mul(self, rhs: ThreeVec) -> f64 {
        self.0*rhs.0 + self.1*rhs.1 + self.2*rhs.2
    }
}

impl ops::Mul<f64> for ThreeVec {

    type Output = ThreeVec;

    fn mul(self, rhs: f64) -> ThreeVec {
        ThreeVec(self.0*rhs, self.1*rhs, self.2*rhs)
    }

}

impl ops::Div<f64> for ThreeVec {

    type Output = ThreeVec;

    fn div(self, rhs: f64) -> ThreeVec {
        ThreeVec(self.0/rhs, self.1/rhs, self.2/rhs)
    }

}

impl ops::Add for ThreeVec {
   
    type Output = ThreeVec; 

    fn add(self, rhs: ThreeVec) -> ThreeVec {
        ThreeVec(self.0+rhs.0, self.1+rhs.1, self.2+rhs.2)
    }

}

impl ops::Sub for ThreeVec {
   
    type Output = ThreeVec; 

    fn sub(self, rhs: ThreeVec) -> ThreeVec {
        ThreeVec(self.0-rhs.0, self.1-rhs.1, self.2-rhs.2)
    }

}

#[derive(Clone,Copy,Debug)]
pub struct ColVec(f64,f64,f64) ;

impl ops::Mul<ColVec> for ColVec {
    
    type Output = ColVec;

    fn mul(self, rhs: ColVec) -> ColVec {
        ColVec(self.0*rhs.0,self.1*rhs.1, self.2*rhs.2)
    }

}

#[derive(Clone,Copy)]
pub struct RayT {
    u: ThreeVec,
    v: ThreeVec
}

pub struct SphereT {
    centre: ThreeVec,
    radius: f64,
    radius2: f64,
    colour : ColVec,
}

pub struct IntersectT<'a> {
    dist: f64,
    sphere: &'a SphereT,
}


pub fn trace(sphere: &SphereT, ray: &RayT) -> Option<f64> {

    let D = ray.u - sphere.centre ;
    let D2 = D*D;
    let vD = D*ray.v;

    let disc = vD*vD - D2 + sphere.radius2;

    if disc < 0.0 {return None};

    let disc = f64::sqrt(disc);

    if vD < 0.0 {
        if -vD < disc { return Some(disc-vD); };
        return Some(-(vD+disc));
    } else if vD < disc { return Some(disc-vD); };
    None 
}

pub fn check_spheres<'a>( ray: &RayT, spheres: &'a Vec<SphereT> ) -> Option<IntersectT<'a>> {
    spheres.into_iter().fold(None::<IntersectT>, |ii, s| { 
            let d = trace(&s,&ray); 
            match (d,ii) {
                (None,i@_) => i,
                (Some(dd),None) => Some(IntersectT{dist: dd,sphere: &s}),
                (Some(dd), Some(i)) => if dd < i.dist { Some(IntersectT{dist: dd,sphere: &s}) } else { Some(i) }
                }
    })
}  

pub fn reflect( ii: &IntersectT, ray: &mut RayT ) -> () {
    ray.u = ray.u + ray.v * ii.dist ; //need to impl mul for f64, ThreeVec
    let Nhat = (ray.u - ii.sphere.centre) / ii.sphere.radius ; //and div for f64, ThreeVec
    let vN = Nhat * ray.v ;

    ray.v = ray.v - Nhat*vN*2.0f64;
    ray.u = ray.u + ray.v * 1e-6f64;

}


fn trace_path_int(ray: RayT, spheres: &Vec<SphereT>, max_reflect: u32) -> ColVec {

    match check_spheres(&ray, &spheres) {
        None => ColVec(1.0,1.0,1.0),
        Some(ii) => if max_reflect == 1 { ii.sphere.colour } else { let mut ray2 = ray;  reflect(&ii, &mut ray2); ii.sphere.colour * trace_path_int(ray2,&spheres,max_reflect-1) }
    }    
}


pub fn trace_path(ray: RayT, spheres: &Vec<SphereT>, max_reflect: u32) -> ColVec {
    match check_spheres(&ray, &spheres) {
        None => ColVec(0.0,0.0,0.0),
        Some(ii) => if max_reflect == 1 { ii.sphere.colour } else { let mut ray2 = ray;  reflect(&ii, &mut ray2); ii.sphere.colour * trace_path_int(ray2,&spheres,max_reflect-1) }
    }    
}


pub fn main() {

    args: Vec<_> = std::env::args_os::collect();

    const HEIGHT: i32 = 600i32;
    const WIDTH: i32 = 600i32;
    const MAXREF: i32 = 20;

    let mut rr = RayT{ u: ThreeVec(0.0,0.0,0.0), v: ThreeVec(0.0,0.0,1.0) };
    let sph = vec![ SphereT{centre: ThreeVec(0.0,0.501,1.0), radius: 0.5, radius2: 0.25, colour: ColVec(0.95,0.95,0.95)}, 
                    SphereT{centre: ThreeVec(0.0,-0.501,1.0), radius: 0.5, radius2: 0.25, colour: ColVec(0.95,0.95,0.9) }, 
                    SphereT{centre: ThreeVec(0.0,0.0,-3.0), radius: 1.0, radius2: 1.0, colour: ColVec(0.9,0.95,0.9) } ];
    
    let mut imgdata = Array::<u8, Ix3>::zeros([2*HEIGHT as usize,2*WIDTH as usize,3]);


    imgdata.indexed_iter_mut().tuples::<(_,_,_)>().for_each(
        |col| { let ( ((x,y,_),r),((_,_,_),g),((_,_,_),b)) = col; 
                rr.u.0 = ( 0.5 + x as f64 - HEIGHT as f64) / HEIGHT as f64; 
                rr.u.1 = ( 0.5 * y as f64 - WIDTH as f64) / WIDTH as f64;
                let colour = trace_path(rr, &sph, MAXREF);
                *r = (colour.0 * 255 as f64) as u8;
                *g = (colour.1 * 255 as f64) as u8;
                *b = (colour.2 * 255 as f64) as u8;
        });


    let path = Path::new(r"img.png");
    let file = File::create(path).unwrap();
    let ref mut img = BufWriter::new(file);

    let mut encoder = png::Encoder::new(img,2*HEIGHT as u32,2*WIDTH as u32);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(imgdata.as_slice().unwrap()).unwrap()


}
