use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::parser::{group_by_modules, ipa_state_transition_map, module_string_to_ast};

// Procedural macro to derive the Step and StepNarrow traits and generate a memory-efficient gate.
//
// The goal is to generate a state transition graph and the corresponding `StepNarrow` implementations
// for the IPA protocol. This macro assumes that a complete IPA steps file exists in the repo at the
// location specified as `STEPS_FILE`. The steps file can be generated by running `collect_steps.py`.
//
// The steps file contains a list of narrowed steps, where each line represents a hierarchy of narrowed
// steps delimited by "/". For example, the following lines represent a hierarchy of narrowed steps:
//
//     RootStep                                => 0
//     RootStep/StepA::A1                      => 1
//     RootStep/StepA::A1/StepB::B1            => 2
//     RootStep/StepA::A1/StepB::B2            => 3
//     RootStep/StepC::C1                      => 4
//     RootStep/StepC::C1/StepD::D1            => 5
//     RootStep/StepC::C1/StepD::D1/StepA::A2  => 6
//     RootStep/StepC::C2                      => 7
//
// From these lines, we want to generate StepNarrow implementations for each step.
//
//     impl StepNarrow<StepA> for Compact {
//         fn narrow(&self, step: &StepA) -> Self {
//             Self(match (self.0, step.as_ref()) {
//                 (0, "A1") => 1,
//                 (5, "A2") => 6,
//                 _ => panic!("invalid state transition"),
//             })
//         }
//     }
//     impl StepNarrow<StepB> for Compact {
//         fn narrow(&self, step: &StepB) -> Self {
//             Self(match (self.0, step.as_ref()) {
//                 (1, "B1") => 2,
//                 (1, "B2") => 3,
//                 _ => panic!("invalid state transition"),
//             })
//         }
//     }
//     ...
//
//
// Currently, this derive notation assumes it annotates the `Compact` struct defined in
// `src/protocol/step/compact.rs`. The `Compact` struct is a wrapper around a `u16` value that
// represents the current state of the IPA protocol.
//
// In the future, we might change the macro to annotate each step in the IPA protocol. The macro
// will then generate both `Descriptive` and `Compact` implementations for the step. However, that
// kind of derive macro requires more annotations such as the fully qualified module path of the
// step. This is because there are many locally-defined `Step` enums in IPA, and we need to
// disambiguate them. However, proc macro doesn't receive the fully qualified module path of the
// annotated struct.

/// Generate a state transition graph and the corresponding `StepNarrow` implementations for the
/// IPA protocol.
pub fn expand(item: TokenStream) -> TokenStream {
    // `item` is the `struct Compact(u16)` in AST
    let ast = parse_macro_input!(item as DeriveInput);
    let gate = &ast.ident;
    match &ast.data {
        syn::Data::Struct(_) => {}
        _ => panic!("derive Gate expects a struct"),
    }

    // we omit the fully qualified module path here because we want to be able to test the macro
    // using our own implementations of `Step` and `StepNarrow`.
    let mut expanded = quote!(
        impl Step for #gate {}
    );

    let steps = ipa_state_transition_map();
    let grouped_steps = group_by_modules(&steps);
    let mut reverse_map = Vec::new();
    let mut deserialize_map = Vec::new();

    for (module, steps) in grouped_steps {
        // generate the `StepNarrow` implementation for each module
        let module = module_string_to_ast(&module);
        let states = steps.iter().map(|s| {
            let new_state = &s.name;
            let new_state_id = s.id;
            let previous_state_id = s.get_parent().unwrap().id;
            quote!(
                (#previous_state_id, #new_state) => #new_state_id,
            )
        });
        expanded.extend(quote!(
            impl StepNarrow<#module> for #gate {
                fn narrow(&self, step: &#module) -> Self {
                    Self(match (self.0, step.as_ref()) {
                        #(#states)*
                        _ => static_state_map(self.0, step.as_ref()),
                    })
                }
            }
        ));

        // generate the reverse map for `impl AsRef<str> for Compact`
        // this is used to convert a state ID to a string representation of the state.
        reverse_map.extend(steps.iter().map(|s| {
            let path = &s.path;
            let state_id = s.id;
            quote!(
                #state_id => #path,
            )
        }));

        deserialize_map.extend(steps.iter().map(|s| {
            let path = &s.path;
            let state_id = s.id;
            quote!(
                #path => #state_id,
            )
        }));
    }

    expanded.extend(quote!(
        impl AsRef<str> for #gate {
            fn as_ref(&self) -> &str {
                match self.0 {
                    #(#reverse_map)*
                    _ => static_reverse_state_map(self.0),
                }
            }
        }
    ));

    // replace `u16` with the type acquired from the AST
    expanded.extend(
        quote!(
            impl Compact {
                pub fn deserialize(s: &str) -> Compact {
                    Self(match s {
                        #(#deserialize_map)*
                        _ => static_deserialize_state_map(s),
                    })
                }
            }
        )
        .into_iter(),
    );

    expanded.into()
}
