"""One bounded JSON request. Host implementation lives in the embedded route module."""
import json
import frida
import sys
from pathlib import Path
from connection import BridgeError, Connection, read_session, deadline
from route import Route

def run(request):
    connection = None
    route = None
    response = None
    try:
        if request.get("schema") != 1 or request.get("operation") not in ("targets", "evaluate"):
            raise BridgeError("invalid_input", "Invalid bridge request")
        with deadline(15):
            state = read_session(request["session"])
            Route.validate_binding(state)
            connection = Connection(state)
            route = Route(connection, connection.state, Path(__file__).parent)
            route.open()
            response = {"schema": 1, "ok": True, "value": route.request(request)}
    except frida.OperationCancelledError:
        response = {"schema":1,"ok":False,"error":{"kind":"timeout",
            "certainty":"Unknown" if route and route.submitted else "NotExecuted",
            "message":"Frida operation deadline; no automatic replay"}}
    except BridgeError as error:
        response = {"schema":1,"ok":False,"error":{
            "kind":error.kind,"certainty":error.certainty,"message":str(error)}}
    except Exception as error:
        response = {"schema":1,"ok":False,"error":{
            "kind":"connection","certainty":"Unknown" if route and route.submitted else "NotExecuted",
            "message":type(error).__name__ + ": " + str(error)}}
    finally:
        try:
            with deadline(5):
                if route is not None:
                    route.close()
                if connection is not None:
                    connection.detach()
        except Exception as error:
            # Never force-unload a script whose native callbacks were not acknowledged closed.
            response = {"schema":1,"ok":False,"error":{
                "kind":"connection","certainty":"Unknown",
                "message":"Native cleanup unconfirmed; do not replay. " + str(error)},
                "operation_result":response}
    return response

if __name__ == "__main__":
    try:
        request = json.loads(Path(sys.argv[1]).read_text())
        result = run(request)
    except Exception as error:
        result = {"schema":1,"ok":False,"error":{"kind":"invalid_input",
            "certainty":"NotExecuted","message":type(error).__name__ + ": " + str(error)}}
    print(json.dumps(result, ensure_ascii=False), flush=True)
