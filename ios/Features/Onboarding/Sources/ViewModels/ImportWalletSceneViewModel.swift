import Components
import Foundation
import GemstonePrimitives
import Localization
import NameService
import Primitives
import PrimitivesComponents
import Style
import SwiftUI
import WalletService
internal import func Gemstone.supportsPrivateKeyImport
import enum Keystore.KeystoreImportType
import enum Keystore.Mnemonic

@Observable
@MainActor
final class ImportWalletSceneViewModel {
    private let walletService: WalletService
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
        walletService: WalletService,
        nameService: any NameServiceable,
        type: ImportWalletType,
        onComplete: (@MainActor @Sendable (ImportWalletSceneResult) -> Void)?,
    ) {
        self.walletService = walletService
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
        case let .chain(chain): Asset(chain).name
        }
    }

    var pasteButtonTitle: String { Localized.Common.paste }
    var pasteButtonImage: Image { Images.System.paste }
    var qrButtonTitle: String { Localized.Wallet.scan }
    var qrButtonImage: Image { Images.System.qrCodeViewfinder }
    var alertTitle: String { Localized.Errors.validation("") }

    var chain: Chain? {
        switch type {
        case .multicoin: .none
        case let .chain(chain): chain
        }
    }

    var showImportTypes: Bool { importTypes.count > 1 }
    var importTypes: [WalletImportType] {
        switch type {
        case .multicoin:
            return [.phrase]
        case let .chain(chain):
            if supportsPrivateKeyImport(chain: chain.rawValue) {
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
        if let chain {
            nameRecordViewModel?.resolve(name: newValue, chain: chain)
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
        let recipient: RecipientImport = {
            if let result = nameRecordViewModel?.state.result {
                return RecipientImport(name: result.name, address: result.address)
            }
            return RecipientImport(name: WalletNameGenerator(type: type, walletService: walletService).name, address: trimmedInput)
        }()
        switch importType {
        case .phrase:
            let words = trimmedInput.split(separator: " ").map { String($0) }
            guard try validateForm(type: importType, address: recipient.address, words: words) else {
                return
            }
            switch type {
            case .multicoin:
                try await importWallet(
                    name: recipient.name,
                    keystoreType: .phrase(words: words, chains: AssetConfiguration.allChains),
                )
            case let .chain(chain):
                try await importWallet(
                    name: recipient.name,
                    keystoreType: .single(words: words, chain: chain),
                )
            }
        case .privateKey:
            guard try validateForm(type: importType, address: recipient.address, words: [trimmedInput]) else {
                return
            }
            try await importWallet(name: recipient.name, keystoreType: .privateKey(text: trimmedInput, chain: chain!))
        case .address:
            guard try validateForm(type: importType, address: recipient.address, words: []) else {
                return
            }
            let chain = chain!
            let address = chain.checksumAddress(recipient.address)

            try await importWallet(name: recipient.name, keystoreType: .address(address: address, chain: chain))
        }
    }

    private func importWallet(name: String, keystoreType: KeystoreImportType) async throws {
        let result = try await walletService.loadOrCreateWallet(name: name, type: keystoreType, source: .import)

        switch result {
        case let .new(wallet):
            try await activateWallet(wallet)
            onComplete?(.new(wallet))
        case let .existing(wallet):
            try await activateWallet(wallet)
            isPresentingExistingWalletName = wallet.name
        }
    }

    private func activateWallet(_ wallet: Wallet) async throws {
        walletService.acceptTerms()
        try await walletService.setCurrent(wallet: wallet)
        buttonState = .normal
    }

    private func validateForm(type: WalletImportType, address: String, words: [String]) throws -> Bool {
        switch type {
        case .phrase:
            for word in words {
                if !Mnemonic.isValidWord(word) {
                    throw WalletImportError.invalidSecretPhraseWord(word: word)
                }
            }
            guard Mnemonic.isValidWords(words) else {
                throw WalletImportError.invalidSecretPhrase
            }
        case .privateKey:
            return !words.joined().isEmpty
        case .address:
            guard chain!.isValidAddress(address) else {
                throw WalletImportError.invalidAddress
            }
        }
        return true
    }
}
