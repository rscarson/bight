//! Utilites for plotting data from table slices. Only minimal configuration is supported. The
//! intended usage is previewing the data.

use std::{error::Error, ops::Range, path::Path};

use crate::table::{Table, TableSlice};

use plotters::{coord::types::RangedCoordf64, prelude::*, style::full_palette::PURPLE};
use polyfit::MonomialFit;

#[derive(Debug, thiserror::Error)]
pub enum PlotError<DE: Error + Send + Sync> {
    #[error("Failed to convert cells into numeric values")]
    DataConversionError,
    #[error("At least 2*1 slice is required to plot")]
    SliceSizeError,
    #[error(transparent)]
    DrawingError(#[from] DrawingAreaErrorKind<DE>),
}

const PLOT_COLORS: [RGBColor; 5] = [BLUE, RED, BLACK, GREEN, PURPLE];

/// Plots the data from the slice using straight segments to connect the points.
///
/// Requires a TableSlice to be at least of width 2 and height 1. Interprets the first column as x
/// values, and the rest as y values. Plots y(x) for each column of y values on the same plot. The
/// data is converted to floats with the [`TryFrom<T>`] trait. The errors of conversion are
/// ignored, and 0.0 is used for value conversion of which has failed.
pub fn plot_segments<T, DB, DE>(
    data: TableSlice<'_, impl Table<Item = T>>,
    chart: ChartBuilder<'_, '_, DB>,
) -> Result<(), PlotError<DE>>
where
    f64: TryFrom<T>,
    T: Clone,
    DB: DrawingBackend<ErrorType = DE>,
    DE: Error + Send + Sync,
{
    let data = PlotData::from_slice(data).ok_or(PlotError::SliceSizeError)?;

    log::trace!("Plotting data: {data:?}");

    let mut chart = prepare_chart(chart, &data)?;

    for (column, color) in data.y.into_iter().zip(PLOT_COLORS.iter().cycle()) {
        chart.draw_series(LineSeries::new(
            column
                .into_iter()
                .zip(data.x.iter().copied())
                .map(|(y, x)| (x, y)),
            color,
        ))?;
    }

    Ok(())
}

/// Plots the data and saves it to a file. See [`plot_segments`] for info about plotting.
pub fn plot_segments_to_file<T, U: Table<Item = T>>(
    data: TableSlice<'_, U>,
    path: &Path,
) -> Result<(), PlotError<impl Error + use<T, U>>>
where
    f64: TryFrom<T>,
    T: Clone,
{
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();

    root.fill(&WHITE)?;

    let chart = ChartBuilder::on(&root);

    plot_segments(data, chart)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present()?;
    log::info!("Plot of {data:?} has been saved to {path:?}");

    Ok(())
}

/// Plots the data from the slice, lineary approiximating each data series (y = ax + b), returning the
/// coefficients `(a, b)` for each series on success.
///
/// Requires a TableSlice to be at least of width 2 and height 1. Interprets the first column as x
/// values, and the rest as y values. Plots y(x) for each column of y values on the same plot. The
/// data is converted to floats with the [`TryFrom<T>`] trait. The errors of conversion are
/// ignored, and 0.0 is used for value conversion of which has failed.
pub fn plot_linear<T, DB, DE>(
    data: TableSlice<'_, impl Table<Item = T>>,
    chart: ChartBuilder<'_, '_, DB>,
) -> Result<Vec<(f64, f64)>, PlotError<DE>>
where
    f64: TryFrom<T>,
    T: Clone,
    DB: DrawingBackend<ErrorType = DE>,
    DE: Error + Send + Sync,
{
    let data = PlotData::from_slice(data).ok_or(PlotError::SliceSizeError)?;

    if data.x.len() < 2 {
        return Err(PlotError::SliceSizeError);
    }

    log::trace!("Plotting data: {data:?}");

    let mut chart = prepare_chart(chart, &data)?;

    let mut coefs = Vec::new();

    for (column, color) in data.y.into_iter().zip(PLOT_COLORS.iter().cycle()) {
        let xy_data = data
            .x
            .iter()
            .copied()
            .zip(column.into_iter())
            .collect::<Vec<_>>();

        let fit =
            MonomialFit::new(&xy_data, 1).expect("The fitting cannnot fail with these parameters");

        coefs.push((fit.coefficients()[0], fit.coefficients()[1]));

        chart.draw_series(LineSeries::new(
            (0..=100)
                .map(|d| {
                    let x = (data.x_range.end - data.x_range.start) * d as f64 / 100.0
                        + data.x_range.start;
                    (x, fit.y(x).unwrap())
                })
                .collect::<Vec<_>>()
                .into_iter(),
            color,
        ))?;

        let style: ShapeStyle = <&RGBColor as Into<ShapeStyle>>::into(color).filled();

        chart.draw_series(
            xy_data
                .into_iter()
                .map(|(x, y)| Circle::new((x, y), 3.0, style)),
        )?;
    }

    Ok(coefs)
}

/// Plots the data with linear approximation and saves it to a file. See [`plot_linear`] for info about plotting.
pub fn plot_linear_to_file<T, U: Table<Item = T>>(
    data: TableSlice<'_, U>,
    path: &Path,
) -> Result<Vec<(f64, f64)>, PlotError<impl Error + use<T, U>>>
where
    f64: TryFrom<T>,
    T: Clone,
{
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();

    root.fill(&WHITE)?;

    let chart = ChartBuilder::on(&root);

    let coefs = plot_linear(data, chart)?;

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present()?;
    log::info!("Plot of {data:?} has been saved to {path:?}");

    Ok(coefs)
}

type FloatChartContext<'a, DB> = ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>;
fn prepare_chart<'a, 'b, DB, DE>(
    mut chart: ChartBuilder<'a, 'b, DB>,
    data: &PlotData,
) -> Result<FloatChartContext<'a, DB>, PlotError<DE>>
where
    DB: DrawingBackend<ErrorType = DE>,
    DE: Error + Send + Sync,
{
    let mut chart = chart
        .margin(10)
        .caption("Data preiew", ("sans-serif", 16))
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(data.x_range.clone(), data.y_range.clone())?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .max_light_lines(4)
        .y_desc("y values")
        .x_desc("x values")
        .draw()?;

    Ok(chart)
}

#[derive(Debug, Clone)]
pub struct PlotData {
    x: Vec<f64>,
    y: Vec<Vec<f64>>,
    x_range: Range<f64>,
    y_range: Range<f64>,
}

impl PlotData {
    /// Prepares f64 data for plotting from a table slice. Interprets the first column as x
    /// values, and the rest as y values. Returns None if the slice is less than 2x1. The errors of conversion are
    /// ignored, and 0.0 is used for value conversion of which has failed.
    pub fn from_slice<T>(data: TableSlice<'_, impl Table<Item = T>>) -> Option<Self>
    where
        f64: TryFrom<T>,
        T: Clone,
    {
        let mut data_cols = data.cols();

        let x: Vec<f64> = data_cols
            .next()?
            .into_iter()
            .map(|x: Option<&T>| {
                x.map(|x: &T| x.clone().try_into().unwrap_or_default())
                    .unwrap_or_default()
            })
            .collect();

        let y: Vec<Vec<f64>> = data_cols
            .map(|c| {
                c.into_iter()
                    .map(|x: Option<&T>| {
                        x.map(|x: &T| -> f64 { x.clone().try_into().unwrap_or_default() })
                            .unwrap_or_default()
                    })
                    .collect::<Vec<f64>>()
            })
            .collect::<Vec<_>>();

        let (y_min, y_max) = {
            let mut y = y.iter().flat_map(|v| v.iter());
            let y0 = *y.next()?;
            y.fold((y0, y0), |(mut y_min, mut y_max), &y| {
                if y_min > y {
                    y_min = y
                }
                if y_max < y {
                    y_max = y
                }
                (y_min, y_max)
            })
        };

        let (x_min, x_max) = {
            let mut x = x.iter();
            let x0 = *x.next()?;
            x.fold((x0, x0), |(mut x_min, mut x_max), &x| {
                if x_min > x {
                    x_min = x
                }
                if x_max < x {
                    x_max = x
                }
                (x_min, x_max)
            })
        };

        Some(Self {
            x,
            y,
            x_range: x_min..x_max,
            y_range: y_min..y_max,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_util::*;

    #[test]
    fn plot_to_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();
        let file = tempfile::Builder::new().suffix(".png").tempfile()?;

        plot_segments_to_file(data.full_slice(), file.path())?;

        Ok(())
    }

    #[test]
    #[ignore = "only for manual output inspenction"]
    /// Saves the plot to a file test_plot.png, which is not cleaned up and can be viewed after the test is run
    fn plot_to_real_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();

        let path = std::path::PathBuf::from(format!("{TEST_OUTPUT_PATH}test_plot.png"));

        plot_segments_to_file(data.full_slice(), &path)?;

        Ok(())
    }

    #[test]
    #[ignore = "only for manual output inspenction"]
    /// Saves the plot to a file test_plot.png, which is not cleaned up and can be viewed after the test is run
    fn plot_linear_to_real_png_file() -> Result<(), Box<dyn Error>> {
        let data = normal_float_data_table();

        let path = std::path::PathBuf::from(format!("{TEST_OUTPUT_PATH}test_plot_linear.png"));

        let coefs = plot_linear_to_file(data.full_slice(), &path)?;

        eprintln!("Linear coefs in test: {coefs:?}");

        Ok(())
    }
}
