// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "TimerBar",
    platforms: [.macOS(.v13)],
    products: [
        .executable(name: "TimerBar", targets: ["TimerBar"])
    ],
    dependencies: [
        .package(url: "https://github.com/stephencelis/SQLite.swift.git", from: "0.15.0")
    ],
    targets: [
        .executableTarget(
            name: "TimerBar",
            dependencies: [
                .product(name: "SQLite", package: "SQLite.swift")
            ]
        )
    ]
)
