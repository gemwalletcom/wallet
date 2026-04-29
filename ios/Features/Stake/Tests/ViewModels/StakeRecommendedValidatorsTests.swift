// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Stake
import Testing

struct StakeRecommendedValidatorsTests {
    @Test func testValidatorsSet() {
        let model = StakeRecommendedValidators()

        #expect(model.validatorsSet(chain: .bitcoin).isEmpty)
        #expect(model.validatorsSet(chain: .cosmos).isEmpty == false)
    }

    @Test
    func randomValidatorPrefersRecommendedMatch() throws {
        let model = StakeRecommendedValidators()
        let recommendedId = try #require(model.validatorsSet(chain: .cosmos).first)
        let validators = [
            DelegationValidator.mock(.cosmos, id: "other"),
            DelegationValidator.mock(.cosmos, id: recommendedId),
        ]

        let selected = try #require(model.randomValidator(chain: .cosmos, from: validators))

        #expect(selected.id == recommendedId)
    }

    @Test
    func randomValidatorFallsBackToTopValidator() throws {
        let model = StakeRecommendedValidators()
        let validators = [
            DelegationValidator.mock(.bitcoin, id: "first", apr: 12),
            DelegationValidator.mock(.bitcoin, id: "second", apr: 8),
        ]

        let selected = try #require(model.randomValidator(chain: .bitcoin, from: validators))

        #expect(selected.id == "first")
    }
}
