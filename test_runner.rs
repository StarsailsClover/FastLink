// FastLink 功能测试运行器
// 编译运行: rustc --edition 2021 test_runner.rs -L target/release/deps && ./test_runner

use std::process::Command;
use std::time::{Duration, Instant};

fn main() {
    println!("========================================");
    println!("   FastLink 功能测试");
    println!("========================================\n");

    let mut results: Vec<TestResult> = Vec::new();

    // 测试 1: 编译测试
    results.push(test_compile());

    // 测试 2: CLI 帮助
    results.push(test_cli_help());

    // 测试 3: 库单元测试
    results.push(test_unit_tests());

    // 打印报告
    print_report(&results);
}

#[derive(Debug)]
struct TestResult {
    name: String,
    passed: bool,
    duration: Duration,
    output: String,
}

fn test_compile() -> TestResult {
    println!("[TEST 1] 编译测试...");
    let start = Instant::now();

    let output = Command::new("cargo")
        .args(&["check", "--workspace"])
        .current_dir(".")
        .output();

    let duration = start.elapsed();

    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            if result.status.success() {
                println!("  ✅ 编译成功");
                TestResult {
                    name: "编译测试".to_string(),
                    passed: true,
                    duration,
                    output: "Workspace 编译成功".to_string(),
                }
            } else {
                println!("  ❌ 编译失败");
                TestResult {
                    name: "编译测试".to_string(),
                    passed: false,
                    duration,
                    output: stderr.to_string(),
                }
            }
        }
        Err(e) => {
            println!("  ❌ 编译命令执行失败: {}", e);
            TestResult {
                name: "编译测试".to_string(),
                passed: false,
                duration,
                output: e.to_string(),
            }
        }
    }
}

fn test_cli_help() -> TestResult {
    println!("[TEST 2] CLI 帮助测试...");
    let start = Instant::now();

    // 检查 CLI 可执行文件是否存在
    let cli_path = "target/release/fastlink-cli.exe";
    let cli_exists = std::path::Path::new(cli_path).exists();

    let duration = start.elapsed();

    if cli_exists {
        println!("  ✅ CLI 可执行文件存在");
        TestResult {
            name: "CLI 可执行文件".to_string(),
            passed: true,
            duration,
            output: format!("找到: {}", cli_path),
        }
    } else {
        println!("  ⚠️ CLI 可执行文件不存在，需要构建");
        TestResult {
            name: "CLI 可执行文件".to_string(),
            passed: false,
            duration,
            output: "需要先运行: cargo build --release".to_string(),
        }
    }
}

fn test_unit_tests() -> TestResult {
    println!("[TEST 3] 单元测试...");
    let start = Instant::now();

    let output = Command::new("cargo")
        .args(&["test", "--lib", "--workspace", "--", "--test-threads=1"])
        .current_dir(".")
        .output();

    let duration = start.elapsed();

    match output {
        Ok(result) => {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            // 解析测试结果
            let passed = stderr.contains("test result: ok");
            
            if passed {
                println!("  ✅ 单元测试通过");
            } else {
                println!("  ⚠️ 部分测试失败或编译错误");
            }

            TestResult {
                name: "单元测试".to_string(),
                passed,
                duration,
                output: format!("{}", stderr),
            }
        }
        Err(e) => {
            println!("  ❌ 测试命令执行失败: {}", e);
            TestResult {
                name: "单元测试".to_string(),
                passed: false,
                duration,
                output: e.to_string(),
            }
        }
    }
}

fn print_report(results: &[TestResult]) {
    println!("\n========================================");
    println!("   测试报告");
    println!("========================================\n");

    let total = results.len();
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = total - passed;
    let total_duration: Duration = results.iter().map(|r| r.duration).sum();

    println!("总测试数: {}", total);
    println!("通过: {}", passed);
    println!("失败: {}", failed);
    println!("成功率: {:.1}%", (passed as f64 / total as f64) * 100.0);
    println!("总耗时: {:?}", total_duration);

    println!("\n详细结果:");
    for result in results {
        let status = if result.passed { "✅ PASS" } else { "❌ FAIL" };
        println!("\n  [{}] {}", status, result.name);
        println!("      耗时: {:?}", result.duration);
        if !result.output.is_empty() {
            let preview = if result.output.len() > 200 {
                format!("{}...", &result.output[..200])
            } else {
                result.output.clone()
            };
            println!("      输出: {}", preview);
        }
    }

    println!("\n========================================");
    if failed == 0 {
        println!("   🎉 所有测试通过！");
    } else {
        println!("   ⚠️  {} 个测试需要关注", failed);
    }
    println!("========================================");
}
