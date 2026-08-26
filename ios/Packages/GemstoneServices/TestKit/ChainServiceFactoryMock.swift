// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import Primitives

public final class ChainServiceFactoryMock: ChainServiceFactorable, Sendable {
    private let chainService: any ChainServiceable

    public init(chainService: any ChainServiceable = ChainServiceMock()) {
        self.chainService = chainService
    }

    public func service(for _: Chain) -> any ChainServiceable {
        chainService
    }
}
