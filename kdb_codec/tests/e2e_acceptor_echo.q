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

assertIpcEq:{[name; sent; recv]
  if[not (-8!sent) ~ -8!recv;
    -2 "assert failed: ", name;
    -2 "  sent(-8!): ", -3!(-8!sent);
    -2 "  recv(-8!): ", -3!(-8!recv);
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

  connStr:raze (":",host,":",string port,":",user,":",pass);
  conn:hsym `$connStr;

  / retry connect (bounded)
  tryConnect:{[c]
    @[hopen; c; {()}]
   };

  h:();
  i:0;
  while[()~h & i<100;
    h:tryConnect conn;
    i+:1;
    if[()~h; system "sleep 0.05"];
   ];
  if[()~h; -2 "failed to connect to ", string conn; '"connect failed"];

  send:{[hh; x] hh (`echo; x)};

  / ---- Atoms ----
  assertEq["null"; (::); send[h; (::)]];
  assertEq["bool"; 1b; send[h; 1b]];
  assertEq["guid"; ("G"$"01020304-0506-0708-090a-0b0c0d0e0f10"); send[h; ("G"$"01020304-0506-0708-090a-0b0c0d0e0f10")]];
  assertEq["byte"; 0x9e; send[h; 0x9e]];
  assertEq["short"; -17h; send[h; -17h]];
  assertEq["int"; -256i; send[h; -256i]];
  assertEq["long"; 86400000000000j; send[h; 86400000000000j]];
  assertEq["real"; 0.25e; send[h; 0.25e]];
  assertEq["float"; 113.0456; send[h; 113.0456]];
  / char atom (type -10). NOTE: "r" is a char vector (string).
  assertEq["char"; first "r"; send[h; first "r"]];
  assertEq["symbol"; `Jordan; send[h; `Jordan]];
  assertEq["string"; "super"; send[h; "super"]]; / type 10

  assertEq["timestamp"; 2019.05.09D00:39:02.000194756; send[h; 2019.05.09D00:39:02.000194756]];
  assertEq["month"; 2019.12m; send[h; 2019.12m]];
  assertEq["date"; 2012.03.12; send[h; 2012.03.12]];
  assertEq["datetime"; 2013.01.10T00:09:50.038; send[h; 2013.01.10T00:09:50.038]];
  assertEq["timespan"; 1D04:34:59.277539844; send[h; 1D04:34:59.277539844]];
  assertEq["minute"; 00:42; send[h; 00:42]];
  assertEq["second"; 00:00:37; send[h; 00:00:37]];
  assertEq["time"; 00:00:12.345; send[h; 00:00:12.345]];

  / ---- Null/Inf/NInf samples ----
  assertEq["int null"; 0Ni; send[h; 0Ni]];
  assertEq["long inf"; 0W; send[h; 0W]];
  assertEq["float null"; 0n; send[h; 0n]];
  assertEq["float inf"; 0w; send[h; 0w]];
  assertEq["float ninf"; -0w; send[h; -0w]];
  assertEq["timestamp null"; 0Np; send[h; 0Np]];
  assertEq["timespan null"; 0Nn; send[h; 0Nn]];

  / ---- Lists ----
  assertEq["bool list"; 101b; send[h; 101b]];
  assertEq["byte list"; 0x00019eff; send[h; 0x00019eff]];
  assertEq["short list"; -3 0 7 12h; send[h; -3 0 7 12h]];
  assertEq["int list"; -256 0 3 1024i; send[h; -256 0 3 1024i]];
  assertEq["long list"; 0 1 2 3 4; send[h; 0 1 2 3 4]];
  assertEq["real list"; 30.2 5.002e; send[h; 30.2 5.002e]];
  assertEq["float list"; 100.23 0.4268 15.882; send[h; 100.23 0.4268 15.882]];
  assertEq["symbol list"; `a`b`c; send[h; `a`b`c]];
  / attribute-bearing list
  assertEq["sorted int list"; `s#1 2 3i; send[h; `s#1 2 3i]];

  / ---- Compound list ----
  assertEq["compound list"; (`alpha; 42i; "bravo"; 1.25); send[h; (`alpha; 42i; "bravo"; 1.25)]];

  / ---- Dictionary ----
  assertEq["dictionary"; (20 30 40i)!001b; send[h; (20 30 40i)!001b]];

  / ---- Table ----
  assertEq["table"; ([] a:10 20 30i; b:`honey`sugar`maple; c:001b); send[h; ([] a:10 20 30i; b:`honey`sugar`maple; c:001b)]];
  / table with attribute
  assertEq["sorted table"; `s#([] a:10 20 30i; b:`honey`sugar`maple; c:001b); send[h; `s#([] a:10 20 30i; b:`honey`sugar`maple; c:001b)]];

  / ---- Keyed table ----
  assertEq["keyed table"; ([a:10 20i] b:100 200i); send[h; ([a:10 20i] b:100 200i)]];

  / ---- Functions ----
  f:{x+y};
  assertIpcEq["lambda root"; f; send[h; f]];
  .d.g:{x+y};
  assertIpcEq["lambda non-root context"; .d.g; send[h; .d.g]];

  / primitive functions (roundtrip by IPC bytes)
  assertIpcEq["unary primitive neg"; value "neg"; send[h; value "neg"]];
  assertIpcEq["binary primitive +"; value "+"; send[h; value "+"]];

  / projections (roundtrip by IPC bytes)
  assertIpcEq["projection (1+)"; (1+); send[h; (1+)]];
  assertIpcEq["projection (+[;2])"; (+[;2]); send[h; (+[;2])]];

  hclose h;
  -1 "ok";
  ::
 };

@[run; (); {-2 "e2e script error: ", x; '"e2e failed"}];

