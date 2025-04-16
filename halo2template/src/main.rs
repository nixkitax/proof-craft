use halo2_proofs::{
    circuit::{Layouter, Region, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Instance, Selector},
    poly::Rotation,
    pasta::Fp,
    dev::MockProver,
};

// Definiamo il circuito con due input privati a e b.
#[derive(Default)]
struct MyCircuit {
    a: Value<Fp>, // input privato a
    b: Value<Fp>, // input privato b
}

// Configurazione delle colonne del circuito (3 colonne di Advice, 1 di Instance, 1 Selector).
#[derive(Clone)]
struct MyConfig {
    col_a: Column<Advice>,
    col_b: Column<Advice>,
    col_c: Column<Advice>,
    instance: Column<Instance>,
    selector: Selector,
}


impl Circuit<Fp> for MyCircuit {
    type Config = MyConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        // Circuito senza testimoni (usato durante la key generation)
        MyCircuit {
            a: Value::unknown(),
            b: Value::unknown(),
        }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> MyConfig {
        // Definiamo le colonne
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let instance = meta.instance_column();
        let selector = meta.selector();

        // Abilitiamo la copia (equality) tra la colonna di output e l’istanza pubblica
        meta.enable_equality(col_c);
        meta.enable_equality(instance);

        // Creiamo un vincolo: quando il selector è attivo, imponi a + b = c
        meta.create_gate("add", |meta| {
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            vec![s * (a + b - c)] // vincolo: s*(a + b - c) = 0
        });

        MyConfig { col_a, col_b, col_c, instance, selector }
    }

    fn synthesize(&self, config: MyConfig, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        // Assegna i valori di a, b, calcola c = a+b e impone i vincoli
        let c_val = self.a.clone().zip(self.b.clone()).map(|(a_val, b_val)| a_val + b_val);
        let mut cell_c = None;
        layouter.assign_region(
            || "witnesses",
            |mut region: Region<'_, Fp>| {
                // Abilita il gate (selector) sulla riga 0
                config.selector.enable(&mut region, 0)?;
                // Assegna i valori privati a e b alla riga 0 delle rispettive colonne
                region.assign_advice(|| "a", config.col_a, 0, || self.a.clone())?;
                region.assign_advice(|| "b", config.col_b, 0, || self.b.clone())?;
                // Assegna il valore c = a + b alla colonna c (riga 0)
                let assigned_c = region.assign_advice(|| "c", config.col_c, 0, || c_val)?;
                // Salva la cella di c per vincolarla all'output pubblico
                cell_c = Some(assigned_c.cell());
                Ok(())
            },
        )?;
        // Vincola la cella c assegnata all'istanza pubblica (indice 0)
        layouter.constrain_instance(cell_c.unwrap(), config.instance, 0)?;
        Ok(())
    }
}

fn main() {
    // Il numero di righe del circuito non può superare 2^k. Usiamo k=4 (16 righe) per sicurezza&#8203;:contentReference[oaicite:0]{index=0}.
    let k = 4;
    // Esempio di valori per i test (a e b privati, c pubblico atteso)
    let a_val = Fp::from(5);
    let b_val = Fp::from(7);
    let c_val = a_val + b_val;
    // Istanzia il circuito con gli input privati noti
    let circuit = MyCircuit {
        a: Value::known(a_val),
        b: Value::known(b_val),
    };
    // Prepara il vettore di input pubblici (qui solo c)
    let public_inputs = vec![c_val];
    // Esegui il circuito in un prover di test (MockProver) e verifica i vincoli
    let prover = MockProver::run(k, &circuit, vec![public_inputs]).unwrap();
    assert!(prover.verify().is_ok(), "Il circuito non soddisfa i vincoli!");
    println!("Proof succeeded!");
}
