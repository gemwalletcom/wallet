// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import GemstonePrimitives
import Primitives

public struct AmountValidator: TextValidator {
    private let validators: [any ValueValidator<BigInt>]
    private let decimals: Int

    public init(
        decimals: Int,
        validators: [any ValueValidator<BigInt>],
    ) {
        self.decimals = decimals
        self.validators = validators
    }

    public var id: String {
        "AmountValidator<\(BigInt.self)>"
    }

    public func validate(_ text: String) throws {
        let value = try NumberInput.value(text, decimals: decimals)
        try validators.forEach { try $0.validate(value) }
    }
}

public extension TextValidator where Self == AmountValidator {
    static func assetAmount(
        decimals: Int,
        validators: [any ValueValidator<BigInt>],
    ) -> Self {
        .init(
            decimals: decimals,
            validators: validators,
        )
    }
}
