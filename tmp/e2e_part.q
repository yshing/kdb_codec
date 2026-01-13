/ e2e_acceptor_echo.q
/ Connect to a kdb_codec acceptor that echoes (`echo; x) back to caller.
/
/ Required env vars:
/ - KDBCODEC_E2E_HOST
/ - KDBCODEC_E2E_PORT
/ - KDBCODEC_E2E_USER
/ - KDBCODEC_E2E_PASS

assertEq:{[name; sent; recv]
  if[not sent ~ recv;
    -2 "assert failed: ", name;
    -2 "  sent: ", -3!sent;
    -2 "  recv: ", -3!recv;
    '"assertion failed";
  ];
  ::
 };

run:{
  host:getenv `KDBCODEC_E2E_HOST;
  port:"I"$getenv `KDBCODEC_E2E_PORT;
  user:getenv `KDBCODEC_E2E_USER;
  pass:getenv `KDBCODEC_E2E_PASS;

  if[0=count host; -2 "missing env KDBCODEC_E2E_HOST"; '"missing env"]; 
  if[0=port; -2 "missing/invalid env KDBCODEC_E2E_PORT"; '"missing env"]; 

  conn:hsym `$":",host,":",string port,":",user,":",pass;

  / retry connect (bounded)
  tryConnect:{[c]
    @[hopen; c; {()}]
   };

  h:();
  i:0;
  while[()~h & i<100;
    h:tryConnect conn;
    i+:1;
