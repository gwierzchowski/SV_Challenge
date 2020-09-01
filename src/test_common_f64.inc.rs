    fn compare(precision:&PointHeight, left: &[PointHeight], right: &[PointHeight]) {
        assert_eq!(left.len(), right.len());
        if *precision == 0.0 || STRICT_EQUALITY {
            assert_eq!(left, right);
        } else {
            //let mut diff = 0.0;
            for (i, _r) in left.iter().enumerate() {
                if (left[i] - right[i]).abs() > precision * (left.len() as PointHeight) {
                    assert!(false, "left[{}]={} != right[{}]={}", i, left[i], i, right[i]);
                }
                // if (left[i] - right[i]).abs() > prec {
                //     assert!(false, "left[{}]={} != right[{}]={}", i, left[i], i, right[i]);
                // }
                // diff += (left[i] - right[i]).abs();
                // if diff > prec * (left.len() as PointHeight) {
                //     assert!(false, "left={:?} != right={:?}; diff={}", left, right, diff);
                // }
            }
        }
    }

    #[test]
    fn sv_case_sample() {
        let points = vec![3.0, 1.0, 6.0, 4.0, 8.0, 9.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[4.0, 4.0, 6.0, 6.0, 8.0, 9.0]);
    }

    #[test]
    fn sv_case_mail2() {
        let points = vec![8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[8.0, 8.0, 4.0]);
    }

    #[test]
    fn sv_case_mail3() {
        let points = vec![1.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[3.0, 8.0, 8.0, 3.0]);
    }

    #[test]
    fn sv_case_mail4() {
        let points = vec![8.0, 4.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[8.0, 7.0, 8.0, 8.0, 3.0]);
    }

    #[test]
    fn sv_case_mail5() {
        let points = vec![1.0, 8.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[3.5, 8.0, 8.0, 8.0, 3.5]);
    }

    #[test]
    fn sv_case_sample_prec0() {
        let points = vec![3.0, 1.0, 6.0, 4.0, 8.0, 9.0];
        let mut landscape = Landscape::create(points);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[4.0, 4.0, 6.0, 6.0, 8.0, 9.0]);
    }

    #[test]
    fn sv_case_mail2_prec0() {
        let points = vec![8.0, 8.0, 1.0];
        let mut landscape = Landscape::create_with_precision(points, 0.0);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[8.0, 8.0, 4.0]);
    }

    #[test]
    fn sv_case_mail3_prec0() {
        let points = vec![1.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create_with_precision(points, 0.0);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[3.0, 8.0, 8.0, 3.0]);
    }

    #[test]
    fn sv_case_mail4_prec0() {
        let points = vec![8.0, 4.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create_with_precision(points, 0.0);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[8.0, 7.0, 8.0, 8.0, 3.0]);
    }

    #[test]
    fn sv_case_mail5_prec0() {
        let points = vec![1.0, 8.0, 8.0, 8.0, 1.0];
        let mut landscape = Landscape::create_with_precision(points, 0.0);
        let prec = landscape.precision();
        let result = landscape.rain_uniform(RAIN_DENSITY, true).unwrap();
        compare(&prec, result, &[3.5, 8.0, 8.0, 8.0, 3.5]);
    }
