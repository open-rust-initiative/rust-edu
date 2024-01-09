use masql::parse::Parser;

#[test]
fn test_insert() {
    let mut p = Parser::new();
    let statement = p.parse("
    SELECT DISTINCT SUM(score) AS score, age
        -- test
        FROM students, teachers
        WHERE @age = (2-1) * SUM(score) 
        GROUP BY name, age 
        HAVING age > 14
        ORDER BY name ASC, age DESC;
    ");
    if let Err(e) = statement {
        println!("{}", e);
    } else if let Ok(r) = statement {
        println!("{:?}", r);
    }
}