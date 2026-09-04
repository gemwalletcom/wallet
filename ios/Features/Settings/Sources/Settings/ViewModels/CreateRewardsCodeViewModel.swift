// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import PrimitivesComponents
import protocol Gemstone.GemRewardsServiceProtocol
import GemstonePrimitives
import GemstoneServices

@Observable
@MainActor
final class CreateRewardsCodeViewModel: TextInputViewModelProtocol {
    private let service: any GemRewardsServiceProtocol
    private let wallet: Wallet
    private let onSuccess: (Rewards) -> Void

    var text: String = ""
    var isLoading: Bool = false
    var errorMessage: String?

    init(
        service: any GemRewardsServiceProtocol,
        wallet: Wallet,
        onSuccess: @escaping (Rewards) -> Void,
    ) {
        self.service = service
        self.wallet = wallet
        self.onSuccess = onSuccess
    }

    var title: String {
        Localized.Rewards.nickname
    }

    var placeholder: String {
        Localized.Rewards.username
    }

    var footer: String? {
        Localized.Rewards.CreateReferralCode.info
    }

    var isActionDisabled: Bool {
        text.isEmpty
    }

    func action() async {
        isLoading = true
        let code = text

        do {
            let rewards = try await service.createReferral(wallet: wallet, code: code)
            onSuccess(rewards)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }
}
