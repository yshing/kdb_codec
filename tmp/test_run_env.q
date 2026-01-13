run:{
  host:getenv `KDBCODEC_E2E_HOST;
  port:"I"$getenv `KDBCODEC_E2E_PORT;
  if[0=count host; -2 "missing env"; '"missing env"];
  if[0=port; -2 "missing/invalid env"; '"missing env"];
  -1 "ok";
  ::
 };
