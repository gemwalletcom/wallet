// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Testing

@testable import PrimitivesComponents

struct StateViewTypeTests {
    @Test
    func setErrorIgnoresCancellation() {
        var state: StateViewType<String> = .loading
        state.setError(CancellationError())
        #expect(state.isLoading)
    }

    @Test
    func setErrorAppliesRegularError() {
        var state: StateViewType<String> = .loading
        state.setError(NSError(domain: "Test", code: 1))
        #expect(state.isError)
    }

    @Test
    func setErrorIgnoresURLCancellation() {
        var state: StateViewType<String> = .loading
        state.setError(NSError(domain: NSURLErrorDomain, code: NSURLErrorCancelled))
        #expect(state.isLoading)
    }
}
