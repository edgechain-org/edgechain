import Flutter
import UIKit

public class EdgechainFlutterPlugin: NSObject, FlutterPlugin {
    public static func register(with registrar: FlutterPluginRegistrar) {
        let channel = FlutterMethodChannel(
            name: "edgechain_flutter",
            binaryMessenger: registrar.messenger()
        )
        let instance = EdgechainFlutterPlugin()
        registrar.addMethodCallDelegate(instance, channel: channel)
    }

    public func handle(_ call: FlutterMethodCall, result: @escaping FlutterResult) {
        // FFI calls are handled directly via flutter_rust_bridge — no platform channel needed.
        result(FlutterMethodNotImplemented)
    }
}
