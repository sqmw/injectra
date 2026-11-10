"""Coordinate target verification and channel lifetime; no local OS process assumptions."""
from dataclasses import replace
from errors import BridgeError
from session_store import read_session
from deadline import deadline
from targets import resolve_target
from transport import resolve_transport
from contracts import TargetProvider, TransportConnector

class Connection:
    def __init__(self, state, *, target_provider: TargetProvider | None = None,
                 connector: TransportConnector | None = None):
        # Resolve all declared capabilities before inspecting a process or opening a channel.
        self.target_provider = target_provider if target_provider is not None else resolve_target(state.device)
        transport = connector if connector is not None else resolve_transport(state.connection)
        self.state = replace(state, target=self.target_provider.bind(state.target))
        self.lease = transport.attach(self.state.target)
        self.session = self.lease.session
        self.script = None
        try:
            self.target_provider.verify_attached(self.session, self.state.target)
        except Exception:
            self.lease.close()
            raise

    def load(self, source):
        self.script = self.session.create_script(source, runtime="qjs")
        self.script.load()
        receipt = self.script.exports_sync.identity()
        try:
            self.target_provider.verify_agent(self.state.target, receipt)
        except Exception:
            self.script.unload()
            self.script = None
            raise
        return self.script

    def detach(self):
        if self.script is not None:
            self.script.unload()
            self.script = None
        self.lease.close()
