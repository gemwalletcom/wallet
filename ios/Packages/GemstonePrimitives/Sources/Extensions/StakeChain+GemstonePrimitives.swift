// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.Config
import Foundation
import Primitives

public extension Primitives.StakeChain {
    var lockTime: TimeInterval {
        Double(Config.shared.getStakeConfig(chain: rawValue).timeLock)
    }

    var minAmount: BigInt {
        BigInt(Config.shared.getStakeConfig(chain: rawValue).minAmount)
    }

    var canChangeAmountOnUnstake: Bool {
        Config.shared.getStakeConfig(chain: rawValue).changeAmountOnUnstake
    }

    var usesFreeze: Bool {
        Config.shared.getStakeConfig(chain: rawValue).usesFreeze
    }

    var usesWholeAmounts: Bool {
        Config.shared.getStakeConfig(chain: rawValue).usesWholeAmounts
    }

    var supportClaimAllRewards: Bool {
        Config.shared.getStakeConfig(chain: rawValue).canClaimAllRewards
    }
}
