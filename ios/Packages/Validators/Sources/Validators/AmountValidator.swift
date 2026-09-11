// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import Foundation
import Primitives

public struct AmountValidator: TextValidator {
    private let validators: [any ValueValidator<BigInt>]
    private let formatter: ValueFormatter
    private let decimals: Int

    public init(
        formatter: ValueFormatter,
        decimals: Int,
        validators: [any ValueValidator<BigInt>],
    ) {
        self.formatter = formatter
        self.decimals = decimals
        self.validators = validators
    }

    public var id: String {
        "AmountValidator<\(BigInt.self)>"
    }

    public func validate(_ text: String) throws {
        let value = try formatter.inputNumber(from: text, decimals: decimals)
        try validators.forEach { try $0.validate(value) }
    }
}

public extension TextValidator where Self == AmountValidator {
    static func assetAmount(
        formatter: ValueFormatter = .init(style: .full),
        decimals: Int,
        validators: [any ValueValidator<BigInt>],
    ) -> Self {
        .init(
            formatter: formatter,
            decimals: decimals,
            validators: validators,
        )
    }
}
