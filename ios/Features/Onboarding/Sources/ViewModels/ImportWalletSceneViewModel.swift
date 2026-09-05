import protocol Gemstone.GemNameServiceProtocol
import enum Gemstone.GemWalletImportType
import protocol Gemstone.GemWalletServiceProtocol
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import Preferences

@Observable
@MainActor
final class ImportWalletSceneViewModel {
    private let service: any GemWalletServiceProtocol
    private let preferences: ObservablePreferences
    private let wordSuggester = WordSuggester()
    let type: ImportWalletType

    var input: String = ""
    var wordsSuggestion: [String] = []
    var importType: WalletImportType = .phrase
    let nameRecordViewModel: NameRecordViewModel?
    var buttonState = ButtonState.normal

    var isPresentingScanner = false
    var isPresentingAlertMessage: AlertMessage?
    var isPresentingExistingWalletName: String?

    private let onComplete: (@MainActor @Sendable (ImportWalletSceneResult) -> Void)?

    init(
        service: any GemWalletServiceProtocol,
        preferences: ObservablePreferences,
        nameService: any GemNameServiceProtocol,
        type: ImportWalletType,
        onComplete: (@MainActor @Sendable (ImportWalletSceneResult) -> Void)?,
    ) {
        self.service = service
        self.preferences = preferences
        self.type = type
        self.onComplete = onComplete
        nameRecordViewModel = switch type {
        case .multicoin: nil
        case .chain: NameRecordViewModel(nameService: nameService)
        }
    }

    var title: String {
        switch type {
        case .multicoin: Localized.Wallet.multicoin
        case let .chain(chain): chain.networkName
        }
    }

    var pasteButtonTitle: String {
        Localized.Common.paste
    }

    var pasteButtonImage: Image {
        Images.System.paste
    }

    var qrButtonTitle: String {
        Localized.Wallet.scan
    }

    var qrButtonImage: Image {
        Images.System.qrCodeViewfinder
    }

    var alertTitle: String {
        Localized.Errors.validation("")
    }

    var chain: Chain? {
        switch type {
        case .multicoin: .none
        case let .chain(chain): chain
        }
    }

    var showImportTypes: Bool {
        importTypes.count > 1
    }

    var importTypes: [WalletImportType] {
        switch type {
        case .multicoin:
            return [.phrase]
        case let .chain(chain):
            if chain.isPrivateKeyImportSupported {
                return [.phrase, .privateKey, .address]
            }
            return [.phrase, .address]
        }
    }

    var footerText: String? {
        switch importType {
        case .phrase, .privateKey: .none
        case .address: Localized.Wallet.importAddressWarning
        }
    }

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }

    var shouldProtectInput: Bool {
        switch importType {
        case .phrase, .privateKey: true
        case .address: false
        }
    }
}

// MARK: - Business Logic

extension ImportWalletSceneViewModel {
    func onChangeImportType(_: WalletImportType, _: WalletImportType) {
        input = ""
    }

    func onChangeInput(_: String, newValue: String) {
        wordsSuggestion = wordSuggester.wordSuggestionCalculate(value: newValue)
        switch importType {
        case .address:
            if let chain {
                nameRecordViewModel?.getNameRecord(name: newValue, chain: chain)
            }
        case .phrase, .privateKey:
            nameRecordViewModel?.reset()
        }
    }

    func onSelectActionButton() async {
        buttonState = .loading(showProgress: true)

        do {
            try await importWallet()
        } catch {
            isPresentingAlertMessage = AlertMessage(
                title: alertTitle,
                message: error.localizedDescription,
            )
            buttonState = .normal
        }
    }

    func onSelectScanQR() {
        isPresentingScanner = true
    }

    func onHandleScan(_ result: String) {
        input = result
    }

    func onSelectWord(_ word: String) {
        input = wordSuggester.selectWordCalculate(
            input: input,
            word: word,
        )
    }

    func onPaste() {
        guard let string = UIPasteboard.general.string else {
            UINotificationFeedbackGenerator().notificationOccurred(.error)
            return
        }
        input = string.trim()

        if shouldProtectInput {
            CopyTypeViewModel.clearClipboard()
        }
    }

    func onSelectExistingWalletContinue() {
        onComplete?(.existing)
    }
}

// MARK: - Private

extension ImportWalletSceneViewModel {
    private func importWallet() async throws {
        let trimmedInput = input.trim()
        let recipient: RecipientImport = if let result = nameRecordViewModel?.state.result {
            RecipientImport(name: result.name, address: result.address)
        } else {
            RecipientImport(name: try await service.defaultWalletName(chain: type.chain?.rawValue).name, address: trimmedInput)
        }
        switch importType {
        case .phrase:
            let words = trimmedInput.split(separator: " ").map { String($0) }
            switch type {
            case .multicoin:
                try await importWallet(
                    name: recipient.name,
                    type: .multicoinPhrase(words: words, chains: AssetConfiguration.allChains.map { $0.map() }),
                )
            case let .chain(chain):
                try await importWallet(
                    name: recipient.name,
                    type: .singlePhrase(words: words, chain: chain.map()),
                )
            }
        case .privateKey:
            try await importWallet(name: recipient.name, type: .privateKey(value: trimmedInput, chain: chain!.map()))
        case .address:
            try await importWallet(name: recipient.name, type: .address(address: recipient.address, chain: chain!.map()))
        }
    }

    private func importWallet(name: String, type: GemWalletImportType) async throws {
        let result = try await service.importWallet(name: name, type: type, source: .import)

        switch result {
        case let .new(wallet):
            await activateWallet(wallet)
            onComplete?(.new(wallet))
        case let .existing(wallet):
            await activateWallet(wallet)
            isPresentingExistingWalletName = wallet.name
        }
    }

    private func activateWallet(_ wallet: Wallet) async {
        preferences.acceptTerms()
        do {
            try service.setCurrentWalletId(walletId: wallet.id.id)
        } catch {
            isPresentingAlertMessage = AlertMessage(title: alertTitle, message: error.localizedDescription)
        }
        buttonState = .normal
    }
}
