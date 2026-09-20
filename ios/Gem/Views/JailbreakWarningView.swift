import GemstonePrimitives
import Style
import SwiftUI

struct JailbreakWarningView: View {
    let onIgnore: () -> Void

    var body: some View {
        ContentUnavailableView {
            Label("Security Warning", systemImage: SystemImage.exclamationmarkTriangle)
        } description: {
            Text("Your device appears to be jailbroken. This may put your wallet and funds at risk.")
        } actions: {
            Link("Learn more", destination: AppUrl.docs(.rootedDevice))
            Button("Ignore", action: onIgnore)
                .buttonStyle(.borderedProminent)
        }
    }
}
