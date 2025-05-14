use halo2_proofs::{
    circuit::{AssignedCell, Chip, Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
    dev::MockProver,
};
use halo2curves::bn256::Fr;

/// Config for quantized addition
#[derive(Clone, Debug)]
struct QuantizedConfig {
    col_a: Column<Advice>,
    col_b: Column<Advice>,
    col_sum: Column<Advice>,
    s_add: Selector,
    instance: Column<Instance>,
}

/// The chip that adds two quantized values
struct QuantizedChip<const SCALE: u64> {
    config: QuantizedConfig,
}

impl<const SCALE: u64> Chip<Fr> for QuantizedChip<SCALE> {
    type Config = QuantizedConfig;
    type Loaded = ();
    fn config(&self) -> &Self::Config {
        &self.config
    }
    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<const SCALE: u64> QuantizedChip<SCALE> {
    fn construct(config: QuantizedConfig) -> Self {
        Self { config }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> QuantizedConfig {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_sum = meta.advice_column();
        let instance = meta.instance_column();
        let s_add = meta.selector();

        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_sum);
        meta.enable_equality(instance);

        meta.create_gate("quantized_add", |meta| {
            let q = meta.query_selector(s_add);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let sum = meta.query_advice(col_sum, Rotation::cur());
            vec![q * (a + b - sum)]
        });

        QuantizedConfig {
            col_a,
            col_b,
            col_sum,
            s_add,
            instance,
        }
    }

    fn quantize(x: f64) -> Fr {
        let scale = SCALE as f64;
        let val = (x * scale).round() as i128;
        if val >= 0 {
            Fr::from(val as u64)
        } else {
            -Fr::from((-val) as u64)
        }
    }

    fn load_private_float(
        &self,
        layouter: &mut impl Layouter<Fr>,
        value: f64,
    ) -> Result<AssignedCell<Fr, Fr>, Error> {
        let val = Self::quantize(value);
        layouter.assign_region(
            || "load private float",
            |mut region| region.assign_advice(|| "a", self.config.col_a, 0, || Value::known(val)),
        )
    }

    fn add(
        &self,
        layouter: &mut impl Layouter<Fr>,
        a: AssignedCell<Fr, Fr>,
        b: AssignedCell<Fr, Fr>,
    ) -> Result<AssignedCell<Fr, Fr>, Error> {
        layouter.assign_region(
            || "add",
            |mut region| {
                self.config.s_add.enable(&mut region, 0)?;
                a.copy_advice(|| "a", &mut region, self.config.col_a, 0)?;
                b.copy_advice(|| "b", &mut region, self.config.col_b, 0)?;
                let sum_val = a.value().zip(b.value()).map(|(x, y)| *x + *y);
                region.assign_advice(
                    || "sum",
                    self.config.col_sum,
                    0,
                    || sum_val,                )
            },
        )
    }

    fn expose_public(
        &self,
        layouter: &mut impl Layouter<Fr>,
        cell: AssignedCell<Fr, Fr>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}

#[derive(Default)]
struct QuantizedAddCircuit<const SCALE: u64> {
    a: f64,
    b: f64,
}

impl<const SCALE: u64> Circuit<Fr> for QuantizedAddCircuit<SCALE> {
    type Config = QuantizedConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { a: 0.0, b: 0.0 }
    }

    fn configure(meta: &mut ConstraintSystem<Fr>) -> Self::Config {
        QuantizedChip::<SCALE>::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fr>,
    ) -> Result<(), Error> {
        let chip = QuantizedChip::<SCALE>::construct(config);
        let a_cell = chip.load_private_float(&mut layouter, self.a)?;
        let b_cell = chip.load_private_float(&mut layouter, self.b)?;
        let sum_cell = chip.add(&mut layouter, a_cell, b_cell)?;
        chip.expose_public(&mut layouter, sum_cell, 0)?;
        Ok(())
    }
}

fn main() {
    const SCALE: u64 = 100;
    let a = 3.5;
    let b = -1.25;
    let expected = a + b;
    let expected_field = QuantizedChip::<SCALE>::quantize(expected);

    let circuit = QuantizedAddCircuit::<SCALE> { a, b };
    let public_inputs = vec![vec![expected_field]];
    let k = 4;

    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    assert_eq!(prover.verify(), Ok(()));
    println!("✅ Circuit verified: {} + {} ≈ {}", a, b, expected);
}
