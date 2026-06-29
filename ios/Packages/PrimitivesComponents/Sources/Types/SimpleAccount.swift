// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives

public struct SimpleAccount {
    public let name: String?
    public let chain: Chain
    public let address: String
    public let memo: String?
    public let assetImage: AssetImage?
    public let addressType: AddressType?

    public init(name: String?, chain: Chain, address: String, memo: String? = nil, assetImage: AssetImage?, addressType: AddressType? = nil) {
        self.name = name
        self.chain = chain
        self.address = address
        self.memo = memo
        self.assetImage = assetImage
        self.addressType = addressType
    }
}
