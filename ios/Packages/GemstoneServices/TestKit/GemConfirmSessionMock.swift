// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemConfirmLoad
public import struct Gemstone.GemConfirmLoadOptions
public import struct Gemstone.GemConfirmScreen
public import protocol Gemstone.GemConfirmSessionProtocol

public final class GemConfirmSessionMock: GemConfirmSessionProtocol, @unchecked Sendable {
    private let initialState: GemConfirmLoad
    private let loadResult: Result<GemConfirmLoad, any Error>
    private var loaded: GemConfirmLoad?
    public var onLoad: (@MainActor () -> Void)?

    public init(state: GemConfirmLoad, load: Result<GemConfirmLoad, any Error>) {
        initialState = state
        loadResult = load
    }

    public func screen() -> GemConfirmScreen {
        GemConfirmScreen(phase: .loading, amountFailed: false, hasCriticalWarning: false, failure: nil)
    }

    public func state() async throws -> GemConfirmLoad {
        loaded ?? initialState
    }

    public func load(options _: GemConfirmLoadOptions) async throws -> GemConfirmLoad {
        await onLoad?()
        loaded = try loadResult.get()
        return try loadResult.get()
    }
}
