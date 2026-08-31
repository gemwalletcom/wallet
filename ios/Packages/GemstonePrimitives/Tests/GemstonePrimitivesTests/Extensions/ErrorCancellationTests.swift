// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemServiceError
@testable import GemstonePrimitives
import Testing

struct ErrorCancellationTests {
    @Test func isCancelled() {
        #expect(GemServiceError.Cancelled.isCancelled)
        #expect(CancellationError().isCancelled)
        #expect(NSError(domain: NSURLErrorDomain, code: NSURLErrorCancelled).isCancelled)

        #expect(!GemServiceError.Api(msg: "failed").isCancelled)
        #expect(!NSError(domain: NSURLErrorDomain, code: NSURLErrorTimedOut).isCancelled)
    }
}
