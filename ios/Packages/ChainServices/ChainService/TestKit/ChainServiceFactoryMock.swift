// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import BlockchainTestKit
import ChainService
import Foundation
import Primitives

public final class ChainServiceFactoryMock: ChainServiceFactorable, Sendable {
    private let chainService: any ChainServiceable

    public var requestInterceptor: any RequestInterceptable {
        EmptyRequestInterceptor()
    }

    public init(chainService: any ChainServiceable = ChainServiceMock()) {
        self.chainService = chainService
    }

    public func service(for _: Chain) -> any ChainServiceable {
        chainService
    }
}
