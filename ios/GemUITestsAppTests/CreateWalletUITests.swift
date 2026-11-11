// Copyright (c). Gem Wallet. All rights reserved.

import XCTest

@MainActor
final class CreateWalletUITests: XCTestCase {
    override func setUpWithError() throws {
        try super.setUpWithError()
        continueAfterFailure = false
    }

    func testCreateWalletAndReceiveBitcoin() {
        let app = XCUIApplication()
        setupPermissionHandler()
        app.launch()
        app.logout()

        if app.isOnboarding {
            app.tapCreateWallet()
        }

        let words = creationFlow(app: app, checkShowSecretDataScene: true)

        app.tap("receive_button")
        app.tap("bitcoin")

        let address = app.descendants(matching: .any).matching(NSPredicate(format: "identifier BEGINSWITH %@", "bc1q")).firstMatch
        XCTAssertTrue(address.waitForExistence(timeout: 10), "bitcoin address not shown")
        app.tapMainAction()

        app.tapBack()
        app.tapBack()
        app.tapWalletBar()

        app.tapWalletSettings()

        app.buttons["Show Secret Phrase"].firstMatch.tap()

        app.tapMainAction()

        let displayedWords = app.getWords()
        XCTAssertEqual(words, displayedWords)

        app.tapBack()
        app.tapBack()
        app.tapBack()

        app.tapCreateWallet()

        _ = creationFlow(app: app, checkShowSecretDataScene: false)

        app.tapWalletBar()
        XCTAssertEqual(app.getWalletsCount(), 2)
    }

    private func creationFlow(app: XCUIApplication, checkShowSecretDataScene: Bool) -> [String] {
        app.acceptTerms()

        app.tapMainAction()

        let words = app.getWords()
        if checkShowSecretDataScene {
            app.reopenFromBackground()
            XCTAssertEqual(app.getWords(), words)
        }
        app.tapMainAction()

        if checkShowSecretDataScene {
            app.reopenFromBackground()
            app.tapBack(leaving: "word_option")
            XCTAssertEqual(app.getWords(), words)
        }
        app.tapMainAction()

        app.pickWords(words)
        app.tapMainAction()
        app.skipEnableAuthentication()

        return words
    }
}
