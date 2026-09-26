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
            let rewards = try await service.createReferral(wallet: wallet.toGem(), code: code)
            onSuccess(rewards)
        } catch let error as GemServiceError {
            errorMessage = error.text().text
        } catch {
            debugLog("rewards code error: \(error)")
        }
        isLoading = false
    }
}
