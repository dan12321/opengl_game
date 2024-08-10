use std::{collections::HashMap, fmt::Debug, fs::OpenOptions, io::Read, path::PathBuf};

use image::DynamicImage;
use tracing::{warn, error, debug};

#[derive(Debug)]
pub struct Model {
    pub objects: Vec<Object>,
}

pub struct Object {
    pub name: String,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub material: usize,
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Object {{ name: {:?}, vertices: {:?}, indices: {:?}, material: {:?} }}", self.name, &self.vertices[0..24], &self.indices[0..3], self.material)
    }
}

impl Object {
    pub fn load(
        obj_file: &PathBuf,
        textures: &mut Vec<Texture>,
        materials: &mut Vec<Material>) -> Vec<Self> {
        let dir = obj_file.parent().unwrap();
        let mut file = OpenOptions::new()
            .read(true)
            .open(obj_file)
            .unwrap();

        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();

        let mut objects: Vec<Self> = Vec::new();
        let mut object_name: Option<String> = None;
        let mut pos_verts: Vec<f32> = Vec::new();
        let mut tex_verts: Vec<f32> = Vec::new();
        let mut norm_verts: Vec<f32> = Vec::new();
        let mut indices: Vec<Index> = Vec::new();
        let mut material: Option<usize> = None;

        for line in text.lines() {
            if line.is_empty() || line.starts_with("#") {
                continue;
            }
            let parts: Vec<&str> = line.split(" ").collect();
            match parts[0] {
                "mtllib" => {
                    let mat_file = dir.join(parts[1]);
                    materials.append(&mut Material::load(&mat_file, textures));
                },
                "usemtl" => {
                    for i in 0..materials.len() {
                        if materials[i].name == parts[1] {
                            if let Some(m) = material {
                                warn!("second-material-use");
                            }
                            material = Some(i);
                            break;
                        }
                        error!(line = line, "material-not-loaded");
                    }
                },
                "v" => {
                    let mut elements = parse_elements(&parts[1..4]);
                    pos_verts.append(&mut elements);
                },
                "vt" => {
                    let mut elements = parse_elements(&parts[1..3]);
                    tex_verts.append(&mut elements);
                },
                "vn" => {
                    let mut elements = parse_elements(&parts[1..4]);
                    norm_verts.append(&mut elements);
                },
                "vp" => {
                    warn!(line = line, "parameter-space-not-supported");
                },
                "f" => {
                    for face in &parts[1..4] {
                        // TODO: Technically this could be negative but haven't seen
                        // that in practice, for now just crash
                        let index: Vec<usize> = face.split("/")
                            .map(|s| s.parse().unwrap())
                            .collect();
                        // Assume all 3 specified
                        indices.push(Index {
                            pos: index[0] - 1,
                            tex: index[1] - 1,
                            norm: index[2] - 1,
                        });
                    }

                }
                "o" => {
                    if let Some(on) = object_name {
                        // add object
                        let mat = material.unwrap();
                        // loop over indices and add verts as needed in
                        // correct format for rendering
                        let mut vertices = Vec::with_capacity(indices.len() * 8);
                        let mut render_indices = Vec::with_capacity(indices.len());
                        let mut indices_map: HashMap<Index, u32> = HashMap::with_capacity(indices.len());
                        for index in &indices {
                            if let Some(render_index) = indices_map.get(index) {
                                render_indices.push(*render_index);
                            } else {
                                let render_index = (vertices.len() / 8) as u32;
                                let mut item = vec![
                                    pos_verts[index.pos * 3],
                                    pos_verts[index.pos * 3 + 1],
                                    pos_verts[index.pos * 3 + 2],
                                    norm_verts[index.norm * 3],
                                    norm_verts[index.norm * 3 + 1],
                                    norm_verts[index.norm * 3 + 2],
                                    tex_verts[index.tex * 2],
                                    tex_verts[index.tex * 2 + 1],
                                ];
                                vertices.append(&mut item);
                                render_indices.push(render_index);
                                indices_map.insert(*index, render_index);
                            }
                        }
                        let object = Self {
                            name: on,
                            material: mat,
                            vertices: vertices,
                            indices: render_indices,
                        };
                        objects.push(object);
                    }
                    object_name = Some(parts[1].to_string());
                    // Not sure verts should clear. the indices keep getting
                    // larger
                    indices.clear();
                    material = None;
                },
                _ => {
                    warn!(line = line, "unexpected-obj-line-type");
                },
            }
        }

        if let Some(on) = object_name {
            // add object
            let mat = material.unwrap();
            // loop over indices and add verts as needed in
            // correct format for rendering
            let mut vertices = Vec::with_capacity(indices.len() * 8);
            let mut render_indices = Vec::with_capacity(indices.len());
            let mut indices_map: HashMap<Index, u32> = HashMap::with_capacity(indices.len());
            for index in &indices {
                if let Some(render_index) = indices_map.get(index) {
                    render_indices.push(*render_index);
                } else {
                    let render_index = (vertices.len() / 8) as u32;
                    let mut item = vec![
                        pos_verts[index.pos * 3],
                        pos_verts[index.pos * 3 + 1],
                        pos_verts[index.pos * 3 + 2],
                        norm_verts[index.norm * 3],
                        norm_verts[index.norm * 3 + 1],
                        norm_verts[index.norm * 3 + 2],
                        tex_verts[index.tex * 2],
                        tex_verts[index.tex * 2 + 1],
                    ];
                    vertices.append(&mut item);
                    render_indices.push(render_index);
                    indices_map.insert(*index, render_index);
                }
            }
            let object = Self {
                name: on,
                material: mat,
                vertices: vertices,
                indices: render_indices,
            };
            objects.push(object);
        }

        objects
    }
}

fn parse_elements(parts: &[&str]) -> Vec<f32> {
    parts.iter()
        .map(|p| p.parse().unwrap())
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Index {
    pos: usize,
    tex: usize,
    norm: usize,
}

pub struct Material {
    pub name: String,
    pub ambient: (f32, f32, f32),
    pub diffuse: (f32, f32, f32),
    pub specular_color: (f32, f32, f32),
    pub specular_exponent: f32,
    pub dissolve: f32,
    // transmission_filter: (f32, f32, f32),
    pub optical_density: f32,
    pub diffuse_map: usize,
    // pub normal_map: usize,
    pub specular_map: usize,
    pub illumination_model: IlluminationModel,
}

impl Debug for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Material {{ name: {:?}, ambient: {:?}, diffuse: {:?}, specular_color: {:?} }}", self.name, self.ambient, self.diffuse, self.specular_color)
    }
}

impl Material {
    pub fn load(filename: &PathBuf, textures: &mut Vec<Texture>) -> Vec<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .open(filename)
            .unwrap();
        let mut text = String::new();
        file.read_to_string(&mut text).unwrap();

        let dir = filename.parent().unwrap();

        let mut materials = Vec::new();
        let mut material_name: Option<String> = None;
        let mut ambient: Option<(f32, f32, f32)> = None;
        let mut diffuse: Option<(f32, f32, f32)> = None;
        let mut specular_color: Option<(f32, f32, f32)> = None;
        let mut specular_exponent: Option<f32> = None;
        let mut dissolve: Option<f32> = None;
        // let mut transmission_filter: Option<(f32, f32, f32)> = None;
        let mut optical_density: Option<f32> = None;
        let mut diffuse_map: Option<usize> = None;
        let mut specular_map: Option<usize> = None;
        // let mut normal_map: Option<usize> = None;
        let mut illumination_model: Option<IlluminationModel> = None;

        for line in text.lines() {
            if line.starts_with("#") {
                continue;
            }

            let parts: Vec<&str> = line.split(" ").collect();
            match parts[0] {
                "newmtl" => {
                    if let Some(mn) = material_name.take() {
                        let material = Material {
                            name: mn,
                            ambient: ambient.take().unwrap(),
                            diffuse: diffuse.take().unwrap(),
                            specular_color: specular_color.take().unwrap(),
                            specular_exponent: specular_exponent.take().unwrap(),
                            dissolve: dissolve.take().unwrap(),
                            // transmission_filter: transmission_filter.take().unwrap(),
                            optical_density: optical_density.take().unwrap(),
                            diffuse_map: diffuse_map.take().unwrap(),
                            specular_map: specular_map.take().unwrap(),
                            // normal_map: normal_map.take().unwrap(),
                            illumination_model: illumination_model.take().unwrap(),
                        };
                        materials.push(material);
                    }
                    material_name = Some(parts[1].to_string());
                },
                "Ka" => {
                    let rbg = parse_elements(&parts[1..4]);
                    ambient = Some((rbg[0], rbg[1], rbg[2]));
                },
                "Kd" => {
                    let rbg = parse_elements(&parts[1..4]);
                    diffuse = Some((rbg[0], rbg[1], rbg[2]));
                },
                "Ks" => {
                    let rbg = parse_elements(&parts[1..4]);
                    specular_color = Some((rbg[0], rbg[1], rbg[2]));
                },
                "Ns" => {
                    let exp: f32 = parts[1].parse().unwrap();
                    specular_exponent = Some(exp);
                },
                "d" => {
                    let dis: f32 = parts[1].parse().unwrap();
                    dissolve = Some(dis);
                },
                "Tr" => {
                    let tr: f32 = parts[1].parse().unwrap();
                    dissolve = Some(1.0 - tr);

                },
                // "Tf" => {
                //     let rbg = parse_elements(&parts[1..4]);
                //     transmission_filter = Some((rbg[0], rbg[1], rbg[2]));
                // },
                "Ni" => {
                    let ni: f32 = parts[1].parse().unwrap();
                    optical_density = Some(ni);
                },
                "illum" => {
                    illumination_model = Some(IlluminationModel::parse(&parts[1]))
                },
                "map_Kd" => {
                    let map_file = dir.join(parts[1]);
                    diffuse_map = Some(load_or_use_texture(textures, map_file));
                },
                "map_Ks" => {
                    let map_file = dir.join(parts[1]);
                    specular_map = Some(load_or_use_texture(textures, map_file));
                },
                "map_Bump" => {
                    let map_file = dir.join(parts[1]);
                    // normal_map = Some(load_or_use_texture(textures, map_file));
                },
                "" => (),
                _ => {
                    warn!(line = line, "unexpected-mtl-line-type");
                }
            }
        }
        if let Some(mn) = material_name.take() {
            let material = Material {
                name: mn,
                ambient: ambient.take().unwrap(),
                diffuse: diffuse.take().unwrap(),
                specular_color: specular_color.take().unwrap(),
                specular_exponent: specular_exponent.take().unwrap(),
                dissolve: dissolve.take().unwrap(),
                // transmission_filter: transmission_filter.take().unwrap(),
                optical_density: optical_density.take().unwrap(),
                diffuse_map: diffuse_map.take().unwrap(),
                specular_map: specular_map.take().unwrap(),
                // normal_map: normal_map.take().unwrap(),
                illumination_model: illumination_model.take().unwrap(),
            };
            materials.push(material);
        }
        materials
    }
}

fn load_or_use_texture(textures: &mut Vec<Texture>, file: PathBuf) -> usize {
    let file_name = file.to_str().unwrap().to_string();
    for i in 0..textures.len() {
        if &textures[i].name == &file_name {
            return i;
        }
    }
    let texture = image::open(&file).unwrap();
    let index = textures.len();
    textures.push(Texture {
        name: file_name,
        image: texture,
    });
    index
}

#[derive(Debug)]
pub struct Texture {
    pub name: String,
    pub image: DynamicImage,
}

#[derive(Debug)]
enum IlluminationModel {
    ColorOnAmbientOff,
    ColorOnAmbientOn,
    HighlightOn,
    ReflectionOnAndRayTraceOn,
    TransparencyGlassOnReflectionRayTraceOn,
    ReflectionFresnelOnAndRayTraceOn,
    TransparencyRefractionOnReflectionFresnelOffRayTraceOn,
    TransparencyRefractionOnReflectionFresnelOnRayTraceOn,
    ReflectionOnRayTraceOff,
    TransparencyGlassOnReflectionRayTraceOff,
    CastsShadowsOntoInvisibleSurfaces,
}

impl IlluminationModel {
    fn parse(value: &str) -> Self {
        match value {
            "0" => Self::ColorOnAmbientOff,
            "1" => Self::ColorOnAmbientOn,
            "2" => Self::HighlightOn,
            "3" => Self::ReflectionOnAndRayTraceOn,
            "4" => Self::TransparencyGlassOnReflectionRayTraceOn,
            "5" => Self::ReflectionFresnelOnAndRayTraceOn,
            "6" => Self::TransparencyRefractionOnReflectionFresnelOffRayTraceOn,
            "7" => Self::TransparencyRefractionOnReflectionFresnelOnRayTraceOn,
            "8" => Self::ReflectionOnRayTraceOff,
            "9" => Self::TransparencyGlassOnReflectionRayTraceOff,
            "10" => Self::CastsShadowsOntoInvisibleSurfaces,
            _ => panic!("unexpected illumination model"),
        }
    }
}