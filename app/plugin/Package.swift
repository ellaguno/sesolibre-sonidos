// swift-tools-version: 5.9
import PackageDescription

// El plugin para iOS. Capacitor (cap sync ios) lo enlaza en CapApp-SPM como
// .package(name: "CapacitorSonidos", path: …) y usa el producto "CapacitorSonidos".
// El núcleo (Rust) llega como XCFramework estático; lo arma scripts/ios-nucleo.sh y no se
// versiona: hay que generarlo antes de abrir el proyecto en Xcode.
let package = Package(
    name: "CapacitorSonidos",
    platforms: [.iOS(.v15)],
    products: [
        .library(
            name: "CapacitorSonidos",
            targets: ["SonidosPlugin"])
    ],
    dependencies: [
        .package(url: "https://github.com/ionic-team/capacitor-swift-pm.git", from: "8.0.0")
    ],
    targets: [
        .binaryTarget(
            name: "SonidosNucleo",
            path: "ios/Frameworks/SonidosNucleo.xcframework"),
        .target(
            name: "SonidosPlugin",
            dependencies: [
                .product(name: "Capacitor", package: "capacitor-swift-pm"),
                .product(name: "Cordova", package: "capacitor-swift-pm"),
                "SonidosNucleo"
            ],
            path: "ios/Sources/SonidosPlugin",
            linkerSettings: [
                .linkedFramework("AVFoundation"),
                .linkedFramework("MediaPlayer")
            ])
    ]
)
