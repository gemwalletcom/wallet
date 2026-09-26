// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemNameRecordState
import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitives
import Primitives
import SwiftUI

@Observable
@MainActor
public final class NameRecordViewModel {
    private let nameService: any GemNameServiceProtocol
    private(set) var nameRecordTask: Task<Void, Never>?

    public var state: GemNameRecordState = .none

    public init(nameService: any GemNameServiceProtocol) {
        self.nameService = nameService
    }

    public func getNameRecord(name: String, chain: Chain) {
        switch nameService.nameInputStep(state: state, name: name, chain: chain.toGem()) {
        case .unchanged:
            return
        case .reset:
            reset()
        case let .resolve(name, debounceMilliseconds):
            nameRecordTask?.cancel()
            state = .loading(name: name, chain: chain.toGem())
            nameRecordTask = Task { await loadNameRecord(name: name, chain: chain, debounceMilliseconds: debounceMilliseconds) }
        }
    }

    private func loadNameRecord(name: String, chain: Chain, debounceMilliseconds: UInt64) async {
        do {
            try await Task.sleep(for: .milliseconds(debounceMilliseconds))
            let resolved = try await nameService.getNameRecord(name: name, chain: chain)
            try Task.checkCancellation()
            state = nameService.resolvedState(state: state, name: name, chain: chain.toGem(), resolved: resolved)
        } catch {
            guard !error.isCancelled else { return }
            state = nameService.resolvedState(state: state, name: name, chain: chain.toGem(), resolved: .error)
        }
    }

    public func reset() {
        nameRecordTask?.cancel()
        state = .none
    }
}
