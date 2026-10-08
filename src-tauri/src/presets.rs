//! Programas populares de diseño, arte digital y 3D listos para añadir.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub name: &'static str,
    pub exe: &'static str,
    pub shortcut: &'static str,
    /// Extensiones de sus archivos de proyecto (para respaldos y verificación).
    pub extensions: &'static [&'static str],
}

const fn preset(name: &'static str, exe: &'static str, extensions: &'static [&'static str]) -> Preset {
    Preset { name, exe, shortcut: "Ctrl+S", extensions }
}

pub fn find(exe: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.exe == exe)
}

pub const PRESETS: &[Preset] = &[
    preset("Blender", "blender.exe", &["blend"]),
    preset("Krita", "krita.exe", &["kra"]),
    preset("Adobe Photoshop", "photoshop.exe", &["psd", "psb"]),
    preset("Adobe Illustrator", "illustrator.exe", &["ai"]),
    preset("Adobe After Effects", "afterfx.exe", &["aep"]),
    preset("Adobe Substance 3D Painter", "adobe substance 3d painter.exe", &["spp"]),
    preset("Clip Studio Paint", "clipstudiopaint.exe", &["clip"]),
    preset("Aseprite", "aseprite.exe", &["aseprite", "ase"]),
    preset("GIMP 2.10", "gimp-2.10.exe", &["xcf"]),
    preset("GIMP 3", "gimp-3.0.exe", &["xcf"]),
    preset("Inkscape", "inkscape.exe", &["svg"]),
    preset("Affinity Photo 2", "photo.exe", &["afphoto"]),
    preset("Affinity Designer 2", "designer.exe", &["afdesign"]),
    preset("Paint Tool SAI 2", "sai2.exe", &["sai2"]),
    preset("MediBang Paint Pro", "medibangpaintpro.exe", &["mdp"]),
    preset("FireAlpaca", "firealpaca.exe", &["mdp"]),
    preset("Paint.NET", "paintdotnet.exe", &["pdn"]),
    preset("OpenToonz", "opentoonz.exe", &["tnz"]),
    preset("ZBrush", "zbrush.exe", &["zpr", "ztl"]),
    preset("Autodesk Maya", "maya.exe", &["ma", "mb"]),
    preset("Autodesk 3ds Max", "3dsmax.exe", &["max"]),
    preset("Cinema 4D", "cinema 4d.exe", &["c4d"]),
    preset("Houdini", "houdinifx.exe", &["hip", "hipnc"]),
];
