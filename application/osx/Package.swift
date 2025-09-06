// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Symposium",
    platforms: [.macOS("15.0")],
    products: [
        .executable(name: "Symposium", targets: ["Symposium"])
    ],
    targets: [
        .executableTarget(
            name: "Symposium",
            dependencies: []
        )
    ]
)
