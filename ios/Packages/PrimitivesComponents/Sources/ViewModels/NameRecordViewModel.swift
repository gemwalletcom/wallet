// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

@Observable
@MainActor
public final class NameRecordViewModel {
    private let nameService: any AddressInputResolving
    private(set) var nameRecordTask: Task<Void, Never>?

    public var state: NameRecordState = .none

    public init(nameService: any AddressInputResolving) {
        self.nameService = nameService
    }

    public func getNameRecord(name: String, chain: Chain) {
        guard name != state.result?.name else { return }
        nameRecordTask?.cancel()

        guard nameService.isNameSupported(name: name) else {
            state = .none
            return
        }

        state = .loading
        nameRecordTask = Task {
            do {
                try await Task.sleep(for: .debounce)
                if let record = try await nameService.getNameRecord(name: name, chain: chain),
                   record.name.isNotEmpty,
                   record.address.isNotEmpty
                {
                    state = .complete(record)
                } else {
                    state = .error
                }
            } catch {
                if !error.isCancelled {
                    state = .error
                }
            }
        }
    }

    public func reset() {
        nameRecordTask?.cancel()
        state = .none
    }

    public func isNameSupported(name: String) -> Bool {
        nameService.isNameSupported(name: name)
    }
}
