(module_declaration
    (module_header 
        (simple_identifier) @module.name)
) @module.item

(ansi_port_declaration
    [
        (net_port_header1
            (port_direction)? @port.direction
            (net_port_type1
                (net_type) @port.class
                (data_type_or_implicit1
                    (implicit_data_type1
                        (packed_dimension)? @port.size)?
                )?    
            )?
        )
        (variable_port_header
            (port_direction)? @port.direction
            (data_type
                (integer_vector_type)? @port.class
                (packed_dimension)? @port.size)
        )
    ]?
    (port_identifier
        (simple_identifier) @port.name)
) @port.item