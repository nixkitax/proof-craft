use halo2_proofs::{
    circuit::{Layouter, Region, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
    pasta::Fp,
    dev::MockProver,
};

// Define the circuit structure with two private inputs: a and b
#[derive(Default)]
struct MyCircuit {
    a: Value<Fp>, // private input a
    b: Value<Fp>, // private input b
}

// Circuit configuration:
// - 3 Advice columns for private and intermediate values
// - 1 Instance column for public output
// - 1 Selector to activate the gate
#[derive(Clone)]
struct MyConfig {
    col_a: Column<Advice>,
    col_b: Column<Advice>,
    col_c: Column<Advice>,
    instance: Column<Instance>,
    selector: Selector,
}

// Implement the circuit: define logic and constraints
impl Circuit<Fp> for MyCircuit {
    type Config = MyConfig;
    type FloorPlanner = SimpleFloorPlanner;

    // Create a circuit without witnesses (used during key generation)
    fn without_witnesses(&self) -> Self {
        MyCircuit {
            a: Value::unknown(),
            b: Value::unknown(),
        }
    }

    // Configure the circuit: define columns and constraints
    fn configure(meta: &mut ConstraintSystem<Fp>) -> MyConfig {
        // Create columns
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let instance = meta.instance_column();
        let selector = meta.selector();

        // Enable equality for the output column and the public instance
        meta.enable_equality(col_c);
        meta.enable_equality(instance);

        // Create a gate: when selector is enabled, enforce a + b = c
        meta.create_gate("add", |meta| {
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            vec![s * (a + b - c)] // constraint: s*(a + b - c) = 0
        });

        MyConfig { col_a, col_b, col_c, instance, selector }
    }

    // Synthesize the circuit: assign values and enforce constraints
    fn synthesize(&self, config: MyConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        // Compute c = a + b
        let c_val = self.a.clone().zip(self.b.clone()).map(|(a_val, b_val)| a_val + b_val);
        let mut cell_c = None;

        layouter.assign_region(
            || "witnesses",
            |mut region: Region<'_, Fp>| {
                // Enable the gate (selector) on row 0
                config.selector.enable(&mut region, 0)?;

                // Assign private inputs a and b to row 0 of their respective columns
                region.assign_advice(|| "a", config.col_a, 0, || self.a.clone())?;
                region.assign_advice(|| "b", config.col_b, 0, || self.b.clone())?;

                // Assign the computed value c = a + b to column c (row 0)
                let assigned_c = region.assign_advice(|| "c", config.col_c, 0, || c_val)?;
                // Save the cell of c to constrain it to the public output
                cell_c = Some(assigned_c.cell());
                Ok(())
            },
        )?;

        // Constrain the assigned cell c to the public instance (index 0)
        layouter.constrain_instance(cell_c.unwrap(), config.instance, 0)?;
        Ok(())
    }
}

fn main() {
    // Define the circuit row capacity (k=4 for 16 rows)
    let k = 4;

    // Example values for testing (private a and b, expected public c)
    let a_val = Fp::from(5);
    let b_val = Fp::from(7);
    let c_val = a_val + b_val;

    // Instantiate the circuit with known private inputs
    let circuit = MyCircuit {
        a: Value::known(a_val),
        b: Value::known(b_val),
    };

    // Prepare public inputs (here only c)
    let public_inputs = vec![c_val];

    // Run the circuit with MockProver and verify constraints
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert!(prover.verify().is_ok(), "Circuit does not satisfy constraints!");
    println!("Proof succeeded!");
}
