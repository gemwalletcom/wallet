// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemRewardsServiceProtocol
import enum Gemstone.GemServiceError
import struct Gemstone.Rewards
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

@Observable
@MainActor
final class RedeemRewardsCodeViewModel: TextInputViewModelProtocol {
    private let service: any GemRewardsServiceProtocol
    private let wallet: Wallet
    private let onSuccess: (String) -> Void

    var text: String
    var isLoading: Bool = false
    var errorMessage: String?

    init(
        service: any GemRewardsServiceProtocol,
        wallet: Wallet,
        code: String = "",
        onSuccess: @escaping (String) -> Void,
    ) {
        self.service = service
        self.wallet = wallet
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
            _ = try await service.useReferralCode(wallet: wallet.toGem(), code: text)
            onSuccess(text)
        } catch let error as GemServiceError {
            errorMessage = error.text().text
        } catch {
            debugLog("rewards code error: \(error)")
        }
        isLoading = false
    }
}
