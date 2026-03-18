use numpy::ndarray::{Array1, Array2, Array3};
use numpy::{PyArray1, PyArray2, PyArray3};
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};

use crate::types::ACHIEVEMENTS;
use crate::{ACTION_NAMES, Env, EnvConfig, GameRules, ITEM_ORDER};

#[pyclass(name = "RustEnv", unsendable)]
pub struct PyRustEnv {
    inner: Env,
}

#[pymethods]
impl PyRustEnv {
    #[new]
    #[pyo3(signature = (area=(64, 64), view=(9, 9), size=(64, 64), reward=true, length=Some(10_000), seed=0, rules_yaml=None))]
    fn new(
        area: (usize, usize),
        view: (usize, usize),
        size: (usize, usize),
        reward: bool,
        length: Option<u32>,
        seed: u64,
        rules_yaml: Option<String>,
    ) -> PyResult<Self> {
        let config = EnvConfig {
            area: [area.0, area.1],
            view: [view.0, view.1],
            size: [size.0, size.1],
            reward,
            length,
            seed,
        };
        let rules = match rules_yaml {
            Some(yaml) => GameRules::from_yaml(&yaml)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e))?,
            None => GameRules::default(),
        };
        Ok(Self {
            inner: Env::with_rules(config, rules),
        })
    }

    #[getter]
    fn action_names(&self) -> Vec<String> {
        ACTION_NAMES.into_iter().map(str::to_string).collect()
    }

    #[getter]
    fn daylight(&self) -> f32 {
        self.inner.world().daylight()
    }

    #[getter]
    fn player_sleeping(&self) -> bool {
        self.inner.player().sleeping()
    }

    #[getter]
    fn player_health(&self) -> i32 {
        self.inner.player().health()
    }

    #[getter]
    fn player_pos<'py>(&self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        position_to_numpy(py, self.inner.player_position())
    }

    fn player_inventory(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        inventory_dict(py, self.inner.player().inventory())
    }

    fn player_achievements(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        achievements_dict(py, self.inner.player().achievements())
    }

    fn reset<'py>(&mut self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        frame_to_numpy(py, &self.inner.reset())
    }

    fn step<'py>(
        &mut self,
        py: Python<'py>,
        action: isize,
    ) -> PyResult<(Py<PyAny>, f32, bool, Py<PyAny>)> {
        let action = usize::try_from(action)
            .map_err(|_| PyIndexError::new_err("action index out of range"))?;
        let result = self
            .inner
            .step_index(action)
            .ok_or_else(|| PyIndexError::new_err("action index out of range"))?;
        let observation = frame_to_numpy(py, &result.observation)?;
        let info = step_info_dict(py, &result.info)?;
        Ok((observation, result.reward, result.done, info))
    }

    #[pyo3(signature = (size=None))]
    fn render<'py>(&self, py: Python<'py>, size: Option<(usize, usize)>) -> PyResult<Py<PyAny>> {
        let size = size.map(|(width, height)| [width, height]);
        frame_to_numpy(py, &self.inner.render(size))
    }
}

pub fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyRustEnv>()?;
    Ok(())
}

fn frame_to_numpy(py: Python<'_>, frame: &crate::Frame) -> PyResult<Py<PyAny>> {
    let array = Array3::from_shape_vec(
        (frame.height(), frame.width(), frame.channels()),
        frame.pixels().to_vec(),
    )
    .expect("frame dimensions should match pixel buffer");
    Ok(PyArray3::from_owned_array(py, array).into_any().unbind())
}

fn semantic_to_numpy(py: Python<'_>, semantic: &crate::SemanticGrid) -> PyResult<Py<PyAny>> {
    let cells = semantic
        .cells()
        .iter()
        .map(|&cell| u8::try_from(cell).expect("semantic ids should fit into u8"))
        .collect();
    let array = Array2::from_shape_vec((semantic.height(), semantic.width()), cells)
        .expect("semantic dimensions should match cell buffer");
    Ok(PyArray2::from_owned_array(py, array).into_any().unbind())
}

fn position_to_numpy(py: Python<'_>, position: crate::Position) -> PyResult<Py<PyAny>> {
    let array = Array1::from_vec(vec![position[0] as i64, position[1] as i64]);
    Ok(PyArray1::from_owned_array(py, array).into_any().unbind())
}

fn inventory_dict(py: Python<'_>, inventory: &crate::Inventory) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    for item in ITEM_ORDER {
        dict.set_item(item.name(), inventory.item(item))?;
    }
    Ok(dict.unbind())
}

fn achievements_dict(
    py: Python<'_>,
    achievements: &crate::AchievementProgress,
) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    for achievement in ACHIEVEMENTS {
        dict.set_item(achievement.name(), achievements.count(achievement))?;
    }
    Ok(dict.unbind())
}

fn step_info_dict(py: Python<'_>, info: &crate::StepInfo) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    dict.set_item("inventory", inventory_dict(py, &info.inventory)?)?;
    dict.set_item("achievements", achievements_dict(py, &info.achievements)?)?;
    dict.set_item("discount", info.discount)?;
    dict.set_item("semantic", semantic_to_numpy(py, &info.semantic)?)?;
    dict.set_item("player_pos", position_to_numpy(py, info.player_pos)?)?;
    dict.set_item("reward", info.reward)?;
    Ok(dict.into_any().unbind())
}
