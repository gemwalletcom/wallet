// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemRewardsServiceProtocol
import enum Gemstone.GemServiceError
import struct Gemstone.Rewards
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
final class RedeemRewardsCodeViewModel: TextInputViewModelProtocol {
    private let service: any GemRewardsServiceProtocol
    private let walletId: WalletId
    private let onSuccess: (Rewards) -> Void

    var text: String
    var isLoading: Bool = false
    var errorMessage: String?

    init(
        service: any GemRewardsServiceProtocol,
        walletId: WalletId,
        code: String = "",
        onSuccess: @escaping (Rewards) -> Void,
    ) {
        self.service = service
        self.walletId = walletId
        text = code
        self.onSuccess = onSuccess
    }

    var title: String {
        Localized.Rewards.referralCode
    }

    var placeholder: String {
        Localized.Rewards.referralCode
    }

    var isActionDisabled: Bool {
        text.isEmpty
    }

    func action() async {
        guard !text.isEmpty else { return }

        isLoading = true
        do {
            let rewards = try await service.useReferralCode(walletId: walletId, code: text)
            onSuccess(rewards)
        } catch let error as GemServiceError {
            errorMessage = error.localizedDescription
        } catch {
            debugLog("rewards code error: \(error)")
        }
        isLoading = false
    }
}
