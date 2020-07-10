use std::result::Result;
use crate::vertex::Vertex;
use crate::maths::Vec3;

enum PLYPropertyType {
	Float,
	UChar,
	UInt,
	Int,
	List(Box<PLYPropertyType>, Box<PLYPropertyType>)
}

impl PLYPropertyType {
	fn from_string(string: String) -> Result<PLYPropertyType, &'static str> {
		println!("ply: from {}", string);
		match string.as_str() {
			"float" => Ok(PLYPropertyType::Float),
			"uchar" => Ok(PLYPropertyType::UChar),
			"uint" => Ok(PLYPropertyType::UInt),
			"int" => Ok(PLYPropertyType::Int),
			_ => {
				Err("Invalid string")
			}
		}
	}
}

#[derive(Clone, Debug)]
enum PLYProperty {
	Float(f32),
	UChar(u8),
	UInt(u32),
	Int(i32),
	List(Box<PLYProperty>, Vec<PLYProperty>),
	UnInit
}

impl PLYProperty {
	fn from_string(property_type: &PLYPropertyType, string: String) -> Result<PLYProperty, &'static str> {
		match &property_type {
			PLYPropertyType::Float => {
				Ok(PLYProperty::Float(string.parse::<f32>().unwrap()))
			}

			PLYPropertyType::UInt => {
				Ok(PLYProperty::UInt(string.parse::<u32>().unwrap()))
			}

			PLYPropertyType::Int => {
				Ok(PLYProperty::Int(string.parse::<i32>().unwrap()))
			}

			PLYPropertyType::UChar => {
				Ok(PLYProperty::UChar(string.parse::<u8>().unwrap()))
			}
		
			_ => {
				Err("Invalid String")
			}
		}
	}
}

struct PLYElementType {
	name: String,
	count: u32,
	property_types: Vec<(String, PLYPropertyType)>,
}

#[derive(Debug)]
struct PLYElement {
	properties: Vec<PLYProperty>
}

pub struct StanfordPLY {
	elements: Vec<Vec<PLYElement>>
}

impl StanfordPLY {
	pub fn new(data: String) -> StanfordPLY {
		
		let mut ply_element_types = Vec::<PLYElementType>::new();

		let mut ply_data = Vec::<Vec<PLYElement>>::new();
//		let mut ply_elements = Vec::<PLYElement>::new();
//
		ply_data.push(Vec::new());
		ply_data.last_mut().unwrap().push(PLYElement {
			properties: Vec::new()
		});

		let mut header = true;

		let mut element_counter = 0u32;
		let mut element_index = 0usize;

		let mut line_number = 0usize;
		let _ = data.lines().for_each(|line| {
				line_number += 1;
				let mut split = line.split(" ");

				if header {
					match split.next().unwrap() {
						"ply" => {

						}

						"format" => {

						}

						"comment" => {
						
						}
					
						"element" => {
							ply_element_types.push(PLYElementType {
								name: split.next().unwrap().to_string(),
								count: split.next().unwrap().parse::<u32>().unwrap(),
								property_types: Vec::new()
							});
						}

						"property" => {
							let next = split.next().unwrap();
							let rhs = match next {
								"list" => PLYPropertyType::List(
									Box::new(PLYPropertyType::from_string(split.next().unwrap().to_string()).unwrap()),
									Box::new(PLYPropertyType::from_string(split.next().unwrap().to_string()).unwrap()),
								),
								_ => {
									PLYPropertyType::from_string(next.to_string()).unwrap()
								}
							};
								println!("ply: not implemented");
							ply_element_types.last_mut().unwrap().property_types.push((split.next().unwrap().to_string(), rhs))
							
						}
						
						"end_header" => {
							header = false;
						}

						_ => {
							println!("ply: {}", line);
						}
					}
				} else {
					let element_type = &ply_element_types[element_index];

					
					for property_type in &element_type.property_types {
						let property = match &property_type.1 {
							PLYPropertyType::List(list_count_type, list_property_type) => {
								let count_property = Box::new(PLYProperty::from_string(&*list_count_type.as_ref(),
										split.next().unwrap().to_string()).unwrap());
								let mut other_properties = Vec::<PLYProperty>::new();

								
								if let PLYProperty::UChar(count) = &*count_property.as_ref() {

									for index in 0..count.clone() {
										other_properties.push(
											PLYProperty::from_string(
												&*list_property_type.as_ref(),
												 split.next().unwrap().to_string()).unwrap()
										)
									}
								} else {
									panic!("PLY: Failed");
								}

								PLYProperty::List(
									count_property,
									other_properties,
								)
							}
							_ => {
								PLYProperty::from_string(&property_type.1, split.next().unwrap().to_string()).unwrap()
							}
						};
						ply_data.last_mut().unwrap()
							.last_mut().unwrap().properties.push(property);
					}


					element_counter += 1;
					if (element_counter == element_type.count) {
						ply_data.push(Vec::new());
						
						element_index += 1;
						element_counter = 0;
					}

					ply_data.last_mut().unwrap().push(PLYElement {
							properties: Vec::new()
						}
					);
				}
			}
		);
		StanfordPLY {elements: ply_data}
	}

	pub fn vertices(&self) -> Vec<Vertex> {
		self.elements[0].iter().map(|element|
			Vertex {
				position: Vec3(
					if let PLYProperty::Float(x) = element.properties[0] {x} else {panic!("unexpected property type")}, 
					if let PLYProperty::Float(y) = element.properties[1] {y} else {panic!("unexpected property type")}, 
					if let PLYProperty::Float(z) = element.properties[2] {z} else {panic!("unexpected property type")}, 
				),
				in_colour: [
					if let PLYProperty::Float(x) = element.properties[0] {0.5 * (x + 1.0)} else {panic!("unexpected property type")}, 
					if let PLYProperty::Float(y) = element.properties[1] {0.5 * (y + 1.0)} else {panic!("unexpected property type")}, 
					if let PLYProperty::Float(z) = element.properties[2] {0.5 * (z + 1.0)} else {panic!("unexpected property type")}, 
				]
			}
		).collect()
	}

	pub fn indices(&self) -> Vec<u32> {
		let mut out = Vec::new();
		for element in &self.elements[1] {
			if let PLYProperty::List(count_property, other_properties) = &element.properties[0] {
				if let PLYProperty::UChar(count) = count_property.as_ref() {
					for i in 0..(*count as usize - 2) {
						match other_properties[0] {
							PLYProperty::UInt(val_0) => {
								if let PLYProperty::UInt(val_1) = other_properties[i + 1] {
									if let PLYProperty::UInt(val_2) = other_properties[i + 2] {
										out.push(val_0);
										out.push(val_1);
										out.push(val_2);
									}
								}
							}

							PLYProperty::Int(val_0) => {
								if let PLYProperty::Int(val_1) = other_properties[i + 1] {
									if let PLYProperty::Int(val_2) = other_properties[i + 2] {
										out.push(val_0 as u32);
										out.push(val_1 as u32);
										out.push(val_2 as u32);
									}
								}
							}
							_ => {
								panic!("unexpected value")
							}
						}
					}
				}
			}
		}
		out
	}

}