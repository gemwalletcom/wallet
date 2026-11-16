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
final class RewardsCodeViewModel: TextInputViewModelProtocol {
    private let type: RewardsCodeType
    private let service: any GemRewardsServiceProtocol
    private let walletId: WalletId
    private let onSuccess: (Rewards) -> Void

    var text: String
    var isLoading: Bool = false
    var errorMessage: String?

    init(
        type: RewardsCodeType,
        service: any GemRewardsServiceProtocol,
        walletId: WalletId,
        onSuccess: @escaping (Rewards) -> Void,
    ) {
        self.type = type
        self.service = service
        self.walletId = walletId
        self.onSuccess = onSuccess
        text = switch type {
        case .create: ""
        case let .redeem(code): code
        }
    }

    var title: String {
        switch type {
        case .create: Localized.Rewards.nickname
        case .redeem: Localized.Rewards.referralCode
        }
    }

    var placeholder: String {
        switch type {
        case .create: Localized.Rewards.username
        case .redeem: Localized.Rewards.referralCode
        }
    }

    var footer: String? {
        switch type {
        case .create: Localized.Rewards.CreateReferralCode.info
        case .redeem: nil
        }
    }

    var isActionDisabled: Bool {
        text.isEmpty
    }

    func action() async {
        if case .redeem = type, text.isEmpty { return }

        isLoading = true
        do {
            let rewards = try await submit(code: text)
            onSuccess(rewards)
        } catch let error as GemServiceError {
            errorMessage = error.localizedDescription
        } catch {
            debugLog("rewards code error: \(error)")
        }
        isLoading = false
    }

    private func submit(code: String) async throws -> Rewards {
        switch type {
        case .create: try await service.createReferral(walletId: walletId, code: code)
        case .redeem: try await service.useReferralCode(walletId: walletId, code: code)
        }
    }
}
