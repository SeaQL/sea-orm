--
-- PostgreSQL database dump
--

-- Dumped from database version 15.6
-- Dumped by pg_dump version 16.1

-- Started on 2024-10-06 12:47:03 CST

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

DROP DATABASE "sea-orm-test";
--
-- TOC entry 3396 (class 1262 OID 185920)
-- Name: sea-orm-test; Type: DATABASE; Schema: -; Owner: postgres
--

CREATE DATABASE "sea-orm-test" WITH TEMPLATE = template0 ENCODING = 'UTF8' LOCALE_PROVIDER = libc LOCALE = 'en_US.utf8';


ALTER DATABASE "sea-orm-test" OWNER TO postgres;

\connect -reuse-previous=on "dbname='sea-orm-test'"

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- TOC entry 838 (class 1247 OID 185922)
-- Name: service_kind; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.service_kind AS ENUM (
    'A',
    'B',
    'C'
);


ALTER TYPE public.service_kind OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 215 (class 1259 OID 185930)
-- Name: vehicle_verify; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.vehicle_verify (
    id bigint NOT NULL,
    service_kinds public.service_kind[] NOT NULL
);


ALTER TABLE public.vehicle_verify OWNER TO postgres;

--
-- TOC entry 214 (class 1259 OID 185929)
-- Name: vehicle_verify_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

ALTER TABLE public.vehicle_verify ALTER COLUMN id ADD GENERATED ALWAYS AS IDENTITY (
    SEQUENCE NAME public.vehicle_verify_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
);


--
-- TOC entry 3390 (class 0 OID 185930)
-- Dependencies: 215
-- Data for Name: vehicle_verify; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.vehicle_verify (id, service_kinds) FROM stdin;
\.


--
-- TOC entry 3397 (class 0 OID 0)
-- Dependencies: 214
-- Name: vehicle_verify_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.vehicle_verify_id_seq', 1, false);


--
-- TOC entry 3246 (class 2606 OID 185936)
-- Name: vehicle_verify vehicle_verify_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.vehicle_verify
    ADD CONSTRAINT vehicle_verify_pkey PRIMARY KEY (id);


-- Completed on 2024-10-06 12:47:04 CST

--
-- PostgreSQL database dump complete
--

