import WebSocket from "ws";

const hostname = '127.0.0.1'
const websocket_port = 8081


const server = new WebSocket.Server({
    port: websocket_port
})
console.log(`started websocket port on ws://${hostname}:${websocket_port}`)
server.on('connection', (ws) => {
    console.log("connection opened")
    ws.on("message", (m) => {
        console.log("got a message",m)
        let parts = m.split(":")
        if(parts[0] === "loaded") {
            ws.send("play:"+parts[1]);
        }
        if(parts[0] === "played") {
            setTimeout(()=>{
                console.log("sending out","pause:"+parts[1])
                ws.send("pause:"+parts[1])
            },4000)
        }
        if(parts[0] === 'paused') {
            ws.send("exit:exit")
        }
    })
    ws.on('close', (code) => {
        console.log("connection closed")
    })

    ws.send("load:examples_music.mp3")

})
server.on("close", (m) => {
    this.log('server closed', m)
})
server.on('error', (e) => {
    this.log("server error", e)
})
