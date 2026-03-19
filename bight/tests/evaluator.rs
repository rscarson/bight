use bight::{evaluator::EvaluatorTable, sync::RcStr, table::Table};

#[test]
#[should_panic]
fn panic_on_non_evaluated() {
    let mut evaluator = EvaluatorTable::default();
    evaluator.set_source((-1, 0), Some("ohno"));
    evaluator.get((-1, 0).into());
}

#[test]
fn string() {
    let mut evaluator = EvaluatorTable::default();
    evaluator.set_source((-1, 0), Some("ohno"));
    evaluator.set_source((0, 0), Some("hi!"));
    evaluator.set_source((1, 0), Some("2"));
    evaluator.set_source((2, 0), Some(r#"\=8"#));
    evaluator.set_source((3, 0), Some(r#"\\ ♥️"#));
    evaluator.set_source((-1, 0), Some("ok"));

    evaluator.evaluate();

    assert_eq!(
        evaluator.get((-1, 0).into()),
        Some(bight::evaluator::TableValue::from_text("ok")).as_ref()
    );
    assert_eq!(
        evaluator.get((0, 0).into()),
        Some(bight::evaluator::TableValue::from_text("hi!")).as_ref()
    );
    assert_eq!(
        evaluator.get((1, 0).into()),
        Some(bight::evaluator::TableValue::from_text("2")).as_ref()
    );
    assert_eq!(
        evaluator.get((2, 0).into()),
        Some(bight::evaluator::TableValue::from_text("=8")).as_ref()
    );
    assert_eq!(
        evaluator.get((3, 0).into()),
        Some(bight::evaluator::TableValue::from_text(r#"\ ♥️"#)).as_ref()
    );
}

#[test]
fn simple_formula() {
    let mut evaluator = EvaluatorTable::default();
    evaluator.set_source((0, 0), Some("=1 + 2"));
    evaluator.set_source((1, 0), Some(r#"="""#));
    evaluator.set_source((2, 0), Some("=(function(x) return x * x end)(3)"));
    evaluator.set_source((3, 0), Some("=nil"));
    evaluator.set_source((4, 0), Some(r#"=1 / 0"#));
    evaluator.set_source((5, 0), Some(r#"=error("this is an error")"#));

    evaluator.evaluate();

    assert_eq!(
        evaluator.get((0, 0).into()),
        Some(bight::evaluator::TableValue::Number(3.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((1, 0).into()),
        Some(bight::evaluator::TableValue::from_text("")).as_ref()
    );
    assert_eq!(
        evaluator.get((2, 0).into()),
        Some(bight::evaluator::TableValue::Number(9.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((3, 0).into()),
        Some(bight::evaluator::TableValue::Empty).as_ref()
    );
    assert_eq!(
        evaluator.get((4, 0).into()),
        Some(bight::evaluator::TableValue::Number(f64::INFINITY)).as_ref()
    );

    let divz = evaluator.get((5, 0).into()).unwrap();

    assert!(divz.is_err(), "{divz}");
    assert!(
        divz.to_string().contains("this is an error"),
        "Error message {divz} did not the expected message"
    );
}

#[test]
fn get_positions() {
    let mut evaluator = EvaluatorTable::default();

    evaluator.set_source((0, 0), Some("=1"));
    evaluator.set_source((1, 0), Some("=A0:val() + 1"));
    evaluator.set_source((2, 2), Some(r#"=GET("B0") + 1"#));
    evaluator.set_source((3, 0), Some("=GET(2, 2) + 1"));

    evaluator.evaluate();

    assert_eq!(
        evaluator.get((0, 0).into()),
        Some(bight::evaluator::TableValue::Number(1.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((1, 0).into()),
        Some(bight::evaluator::TableValue::Number(2.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((2, 2).into()),
        Some(bight::evaluator::TableValue::Number(3.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((3, 0).into()),
        Some(bight::evaluator::TableValue::Number(4.0)).as_ref()
    );
}

#[test]
fn get_relative() {
    let mut evaluator = EvaluatorTable::default();

    evaluator.set_source((0, 0), Some("=1"));
    evaluator.set_source((1, 0), Some("=(POS - B0):val() + 1"));
    evaluator.set_source((2, 2), Some(r#"=(POS - B2):val() + 1"#));
    evaluator.set_source((3, 0), Some("=RELX(-2) + 2"));
    evaluator.set_source((1, 2), Some("=RELY(-2) + 2"));

    evaluator.evaluate();

    assert_eq!(
        evaluator.get((0, 0).into()),
        Some(bight::evaluator::TableValue::Number(1.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((1, 0).into()),
        Some(bight::evaluator::TableValue::Number(2.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((2, 2).into()),
        Some(bight::evaluator::TableValue::Number(3.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((3, 0).into()),
        Some(bight::evaluator::TableValue::Number(4.0)).as_ref()
    );
    assert_eq!(
        evaluator.get((1, 2).into()),
        Some(bight::evaluator::TableValue::Number(4.0)).as_ref()
    );
}

#[test]
fn dependency_cycle() {
    let mut evaluator = EvaluatorTable::default();

    evaluator.set_source((0, 0), Some("=1"));
    evaluator.set_source((1, 0), Some("=A0:val() + D0:val() + 1"));
    evaluator.set_source((2, 2), Some(r#"=GET("B0") + 1"#));
    evaluator.set_source((3, 0), Some("=GET(2, 2) + 1"));

    evaluator.evaluate();

    assert_eq!(
        evaluator.get((0, 0).into()),
        Some(bight::evaluator::TableValue::Number(1.0)).as_ref()
    );

    let dc = evaluator.get((1, 0).into()).unwrap().to_string();
    assert!(
        dc.to_lowercase().contains("dependency cycle"),
        "{dc} doesn't contain \"dependency cycle\"",
    );

    eprintln!("{:?}", evaluator.get((1, 0).into()).unwrap());

    let dc = evaluator.get((2, 2).into()).unwrap().to_string();
    assert!(
        dc.to_lowercase().contains("dependency cycle"),
        "{dc} doesn't contain \"dependency cycle\"",
    );

    let dc = evaluator.get((3, 0).into()).unwrap().to_string();
    assert!(
        dc.to_lowercase().contains("dependency cycle"),
        "{dc} doesn't contain \"dependency cycle\"",
    );
}

#[test]
fn global_is_different() {
    let mut evaluator = EvaluatorTable::default();

    let source: RcStr = r#"=(function()
  if _G.x == 1 then
     error("ohno!!! ")
  else
    _G.x = 1
    x = 1
  end
  return x
end)()"#
        .into();

    for i in 0..10 {
        evaluator.set_source((i, i), Some(source.clone()));
    }

    for _ in 0..1000 {
        evaluator.evaluate();
        for i in 0..10 {
            assert_eq!(
                evaluator.get((i, i).into()),
                Some(bight::evaluator::TableValue::Number(1.0)).as_ref()
            );
        }
    }
}

/// This test is failing to demonstare the problem described in issue #8
#[test]
#[ignore = "fails, see issue #8"]
fn global_math_is_different() {
    let mut evaluator = EvaluatorTable::default();

    let source: RcStr = r#"=(function()
  if _G.math.x == 1 then
     error("ohno!!! ")
  else
    _G.math.x = 1
    math.x = 1
  end
  return math.x
end)()"#
        .into();

    for i in 0..10 {
        evaluator.set_source((i, i), Some(source.clone()));
    }

    for _ in 0..1000 {
        evaluator.evaluate();
        for i in 0..10 {
            assert_eq!(
                evaluator.get((i, i).into()),
                Some(bight::evaluator::TableValue::Number(1.0)).as_ref()
            );
        }
    }
}
