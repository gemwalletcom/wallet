// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.recommendedValidator
import func Gemstone.recommendedValidatorIds
import GemstonePrimitives
import enum Primitives.Chain
import struct Primitives.DelegationValidator

public struct StakeRecommendedValidators {
    public init() {}

    public func validatorsSet(chain: Chain) -> Set<String> {
        Set(recommendedValidatorIds(chain: chain.rawValue))
    }

    public func randomValidator(
        chain: Chain,
        from validators: [DelegationValidator],
    ) -> DelegationValidator? {
        guard let validators = try? validators.map({ try $0.json() }) else { return nil }
        return try? recommendedValidator(chain: chain.rawValue, validators: validators).map { try DelegationValidator($0) }
    }
}
