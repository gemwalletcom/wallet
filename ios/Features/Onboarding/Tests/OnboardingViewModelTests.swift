// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import class Gemstone.GemNameService
import class Gemstone.GemWalletService
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
@testable import Onboarding
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct AcceptTermsViewModelTests {
    @Test
    func theTermsComeFromCoreAndStartUnconfirmed() {
        let model = AcceptTermsViewModel(onNext: nil)

        #expect(model.items.isNotEmpty)
        #expect(model.isConfirmed == false)
        #expect(model.state.isNoData)
    }

    @Test
    func everyTermMustBeTickedBeforeContinuing() {
        let model = AcceptTermsViewModel(onNext: nil)

        for item in model.items.dropLast() {
            item.isConfirmed = true
        }
        #expect(model.isConfirmed == false)

        model.items.forEach { $0.isConfirmed = true }
        #expect(model.isConfirmed)
        #expect(model.state.isNoData == false)
    }
}

struct TermItemViewModelTests {
    @Test
    func tickingATermChangesHowItReads() {
        let item = TermItemViewModel(message: "I understand")

        let unconfirmed = item.style.color
        item.isConfirmed = true

        #expect(item.id == "I understand")
        #expect(item.style.color != unconfirmed)
    }
}

struct SecurityReminderViewModelTests {
    @Test
    func theRemindersComeFromCore() {
        let model = SecurityReminderViewModel(title: "Before you start", onNext: {})

        #expect(model.title == "Before you start")
        #expect(model.items.isNotEmpty)
        #expect(model.message.isNotEmpty)
    }
}

struct NewSecretPhraseViewModelTests {
    @Test
    func continuingHandsBackTheSameWords() {
        let recorder = WordsRecorder()
        let model = NewSecretPhraseViewModel(words: ["alpha", "bravo"], onCreateWallet: { recorder.record($0) })

        model.continueAction?()

        #expect(recorder.words == [["alpha", "bravo"]])
    }
}

struct ImportWalletTypeViewModelTests {
    @Test
    func anEmptyQueryOffersEveryChain() {
        let model = ImportWalletTypeViewModel()

        #expect(model.items(for: "").isNotEmpty)
    }

    @Test
    func aQueryNarrowsTheChains() {
        let model = ImportWalletTypeViewModel()

        let all = model.items(for: "")
        let filtered = model.items(for: "bitcoin")

        #expect(filtered.isNotEmpty)
        #expect(filtered.count < all.count)
        #expect(filtered.contains(.bitcoin))
    }

    @Test
    func anUnknownQueryOffersNothing() {
        #expect(ImportWalletTypeViewModel().items(for: "zzzzzz").isEmpty)
    }
}

@MainActor
struct SetupWalletViewModelTests {
    @Test
    func theSceneStartsFromTheWalletName() {
        let wallet = Primitives.Wallet.mock(name: "Imported")
        let model = SetupWalletViewModel(wallet: wallet, service: GemWalletService.mock(), onSelectImage: { _ in }, onComplete: { _ in })

        #expect(model.nameInput == "Imported")
        #expect(model.title.isNotEmpty)
    }

    @Test
    func renamingAWalletThatIsGoneShowsTheError() async {
        let model = SetupWalletViewModel(
            wallet: .mock(id: .multicoin(address: "0xmissing")),
            service: GemWalletService.mock(),
            onSelectImage: { _ in },
            onComplete: { _ in },
        )
        model.nameInput = "Renamed"

        await model.onChangeWalletName()

        #expect(model.isPresentingAlertMessage != nil)
    }

    @Test
    func finishingHandsBackTheWallet() {
        let recorder = WalletRecorder()
        let wallet = Primitives.Wallet.mock(name: "Imported")
        let model = SetupWalletViewModel(wallet: wallet, service: GemWalletService.mock(), onSelectImage: { _ in }, onComplete: { recorder.record($0) })

        model.onComplete()
        model.onSelectImage()

        #expect(recorder.names == ["Imported"])
    }
}

@MainActor
struct ImportWalletViewModelTests {
    @Test
    func pickingAnImageRemembersTheWallet() {
        let model = ImportWalletViewModel(
            service: GemWalletService.mock(),
            preferences: .mock(),
            nameService: GemNameService.mock(),
            onComplete: nil,
        )

        model.presentSelectImage(wallet: .mock(name: "Imported"))

        #expect(model.isPresentingSelectImageWallet?.name == "Imported")
    }
}

private final class WordsRecorder: @unchecked Sendable {
    private(set) var words: [[String]] = []

    func record(_ value: [String]) {
        words.append(value)
    }
}

private final class WalletRecorder: @unchecked Sendable {
    private(set) var names: [String] = []

    func record(_ wallet: Primitives.Wallet) {
        names.append(wallet.name)
    }
}
