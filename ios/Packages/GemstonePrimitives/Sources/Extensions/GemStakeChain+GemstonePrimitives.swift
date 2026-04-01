// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension StakeChain {
    func map() -> GemStakeChain {
        switch self {
        case .cosmos: .cosmos
        case .osmosis: .osmosis
        case .injective: .injective
        case .sei: .sei
        case .celestia: .celestia
        case .ethereum: .ethereum
        case .solana: .solana
        case .sui: .sui
        case .smartChain: .smartChain
        case .tron: .tron
        case .aptos: .aptos
        case .hyperCore: .hyperCore
        case .monad: .monad
        }
    }

    var lockTime: TimeInterval {
        Double(Config.shared.getStakeConfig(chain: rawValue).timeLock)
    }

    var minAmount: BigInt {
        BigInt(Config.shared.getStakeConfig(chain: rawValue).minAmount)
    }

    var canChangeAmountOnUnstake: Bool {
        Config.shared.getStakeConfig(chain: rawValue).changeAmountOnUnstake
    }

    var supportRedelegate: Bool {
        Config.shared.getStakeConfig(chain: rawValue).canRedelegate
    }

    var supportWidthdraw: Bool {
        Config.shared.getStakeConfig(chain: rawValue).canWithdraw
    }

    var supportClaimRewards: Bool {
        Config.shared.getStakeConfig(chain: rawValue).canClaimRewards
    }
}
