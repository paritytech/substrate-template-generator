# Substrate Node Template Generator

A tool to generate stand-alone node templates of a customized Substrate clients used in
"Substrate Library Extension" (SLE) projects like Cumulus, Canvas, Frontier, as well as
any custom chain that intends for users to build off of as a base template included in their source.

## Expected Project Hierarchy

This tool is tightly coupled with _how_ a SLE is structured. To use this, your project needs to roughly
conform with the style of:
- A local template node that has similar structure to the [Substrate node template](https://github.com/paritytech/substrate/tree/master/bin/node-template)
  - (Optional, encouraged) CI that builds against your template to do integration testing. This also assures the 
- Local pallets defined and added to the template runtime
- Local custom logic, like pallets, contained in crates outside the template to enable sourcing them through cargo from an external project.
- Source available on a public repo that cargo can use (GitHub is the only supported for now)

## Credit

Based heavily on the integrated [Node Template Release Tool]() included in Substrate.