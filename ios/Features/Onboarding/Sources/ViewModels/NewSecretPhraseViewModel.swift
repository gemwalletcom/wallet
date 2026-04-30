import Components
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import WalletService

struct NewSecretPhraseViewModel: SecretPhraseViewableModel {
    private let onCreateWallet: ([String]) -> Void
    let words: [String]

    var calloutViewStyle: CalloutViewStyle? {
        .header(title: Localized.SecretPhrase.savePhraseSafely)
    }

    var continueAction: VoidAction {
        { onCreateWallet(words) }
    }

    init(
        walletService: WalletService,
        onCreateWallet: @escaping (([String]) -> Void),
    ) {
        self.onCreateWallet = onCreateWallet
        do {
            words = try walletService.createWallet()
        } catch {
            fatalError("Unable to create wallet")
        }
    }

    var title: String {
        Localized.Wallet.New.title
    }

    var type: SecretPhraseDataType {
        .words(words: WordIndex.rows(for: words))
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .secretPhrase,
            copyValue: MnemonicFormatter.fromArray(words: words),
        )
    }
}
