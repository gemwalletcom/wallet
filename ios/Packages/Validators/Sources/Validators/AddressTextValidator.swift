// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

public struct AddressTextValidator: TextValidator {
    private let asset: Asset
    private let nameService: any AddressInputResolving

    public init(asset: Asset, nameService: any AddressInputResolving) {
        self.asset = asset
        self.nameService = nameService
    }

    public func validate(_ text: String) throws {
        guard nameService.validateRecipient(chain: asset.chain.rawValue, input: text, nameRecord: nil).isValid else {
            throw TransferError.invalidAddress(asset: asset)
        }
    }

    public var id: String {
        asset.id.identifier
    }
}

public extension TextValidator where Self == AddressTextValidator {
    static func address(_ asset: Asset, nameService: any AddressInputResolving) -> Self {
        .init(asset: asset, nameService: nameService)
    }
}
