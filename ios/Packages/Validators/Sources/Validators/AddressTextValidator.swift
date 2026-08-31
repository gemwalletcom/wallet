// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAddressServiceProtocol
import GemstonePrimitives
import Primitives

public struct AddressTextValidator: TextValidator {
    private let asset: Asset
    private let addressService: any GemAddressServiceProtocol

    public init(asset: Asset, addressService: any GemAddressServiceProtocol) {
        self.asset = asset
        self.addressService = addressService
    }

    public func validate(_ text: String) throws {
        guard asset.chain.isValidAddress(text, addressService: addressService) else {
            throw TransferError.invalidAddress(asset: asset)
        }
    }

    public var id: String {
        asset.id.identifier
    }
}

public extension TextValidator where Self == AddressTextValidator {
    static func address(_ asset: Asset, addressService: any GemAddressServiceProtocol) -> Self {
        .init(asset: asset, addressService: addressService)
    }
}
